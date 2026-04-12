import json
import subprocess
import zipfile
import os
import sys
import requests
import glob
import time
import hashlib
import tempfile
import shutil
import fnmatch
import select
import tty
import termios
from datetime import datetime
from pathlib import Path
from watchdog.observers.polling import PollingObserver as Observer
from watchdog.events import FileSystemEventHandler
from colorama import Fore, init

init(autoreset=True)


def log(msg: str, color: str = Fore.CYAN) -> None:
    print(color + msg)


class Config:
    def __init__(self, path: str = "config.json") -> None:
        if not os.path.exists(path):
            raise FileNotFoundError(f"Config file not found: {path}")
        with open(path, "r") as f:
            self._settings = json.load(f)

    def __getattr__(self, name: str):
        if name.startswith("_"):
            raise AttributeError(name)
        return self._settings[name]


def safe_path(file_path: str, base_path: str) -> bool:
    try:
        return Path(file_path).resolve().is_relative_to(Path(base_path).resolve())
    except (ValueError, OSError):
        return False


def compute_checksum(file_path: str) -> str:
    h = hashlib.sha256()
    try:
        with open(file_path, "rb") as f:
            for chunk in iter(lambda: f.read(8192), b""):
                h.update(chunk)
        return h.hexdigest()
    except IOError:
        return ""


def verify_zip(zip_path: str) -> bool:
    manifest_path = zip_path + ".manifest.json"
    if not os.path.exists(manifest_path):
        return False
    try:
        with open(manifest_path, "r") as f:
            stored = json.load(f).get("checksum", "")
        return stored and compute_checksum(zip_path) == stored
    except (json.JSONDecodeError, IOError):
        return False


def save_manifest(zip_path: str) -> None:
    manifest = {"checksum": compute_checksum(zip_path), "created_at": datetime.now().isoformat()}
    with open(zip_path + ".manifest.json", "w") as f:
        json.dump(manifest, f)


class BackupManager:
    def __init__(self, config: Config) -> None:
        self._config = config
        self._output = os.path.join(config.base_save_folder, config.output_subfolder)
        self._lock_file = os.path.join(self._output, ".lock")
        self._timestamp: datetime | None = None
        self._running = True
        os.makedirs(self._output, exist_ok=True)

    def _get_time(self) -> datetime:
        for i in range(3):
            try:
                return datetime.fromisoformat(requests.get(
                    self._config.time_api_url, timeout=5
                ).json()["dateTime"])
            except Exception:
                if i < 2:
                    time.sleep(2 ** i)
        raise RuntimeError("Failed to fetch time from API")

    def _lock(self) -> bool:
        start = time.time()
        while os.path.exists(self._lock_file):
            if time.time() - start > 30:
                return False
            time.sleep(0.1)
        open(self._lock_file, "w").close()
        return True

    def _unlock(self) -> None:
        if os.path.exists(self._lock_file):
            os.remove(self._lock_file)

    def _files(self) -> list[str]:
        files = []
        for p in self._config.files_to_save:
            matches = glob.glob(os.path.join(self._config.base_save_folder, p))
            if not matches:
                log(f"No files match: {p}", Fore.YELLOW)
            files.extend(m for m in matches if safe_path(m, self._config.base_save_folder))
        return list(set(files))

    def _zip_name(self, timestamp: datetime) -> str:
        return os.path.join(self._output, f"d2backup_{timestamp:%Y%m%d_%H%M}.zip")

    def _create_zip(self, path: str, timestamp: datetime) -> bool:
        files = self._files()
        if not files:
            log("No files to back up.")
            return False
        try:
            with zipfile.ZipFile(path, 'w', zipfile.ZIP_DEFLATED) as zf:
                for f in files:
                    zf.write(f, os.path.basename(f))
            save_manifest(path)
            self._timestamp = timestamp
            log(f"Created: {path}", Fore.GREEN)
            return True
        except IOError as e:
            log(f"Failed to create ZIP: {e}", Fore.RED)
            if os.path.exists(path):
                os.remove(path)
            return False

    def _do_backup(self) -> None:
        if not self._lock():
            log("Backup skipped: lock held", Fore.YELLOW)
            return
        try:
            while True:
                try:
                    timestamp = self._get_time()
                    break
                except RuntimeError:
                    log("Failed to fetch time, retrying in 10s...", Fore.RED)
                    time.sleep(10)
            self._create_zip(self._zip_name(timestamp), timestamp)
            self._cleanup()
        finally:
            self._unlock()

    def _cleanup(self) -> None:
        zips = sorted(
            [f for f in os.listdir(self._output) if f.endswith(".zip")],
            key=lambda x: datetime.strptime(x.replace("d2backup_", "").replace(".zip", ""), "%Y%m%d_%H%M"),
            reverse=True
        )
        for f in zips[self._config.max_backups:]:
            try:
                os.remove(os.path.join(self._output, f))
                os.remove(os.path.join(self._output, f + ".manifest.json"))
                log(f"Deleted: {f}", Fore.YELLOW)
            except OSError:
                pass

    def _restore(self, zip_path: str) -> bool:
        if not verify_zip(zip_path):
            log("Checksum mismatch!", Fore.RED)
            return False
        try:
            with zipfile.ZipFile(zip_path, 'r') as zf:
                for member in zf.namelist():
                    if not safe_path(os.path.join(self._config.base_save_folder, member), self._config.base_save_folder):
                        log(f"Unsafe path: {member}", Fore.RED)
                        return False
            with tempfile.TemporaryDirectory() as tmp:
                with zipfile.ZipFile(zip_path, 'r') as zf:
                    zf.extractall(tmp)
                for member in os.listdir(tmp):
                    shutil.copy2(os.path.join(tmp, member), self._config.base_save_folder)
            log("Restore complete.", Fore.GREEN)
            return True
        except (zipfile.BadZipFile, IOError, OSError) as e:
            log(f"Restore failed: {e}", Fore.RED)
            return False

    def backups(self) -> list[str]:
        return sorted([f for f in os.listdir(self._output) if f.endswith(".zip")], reverse=True)

    def restore_menu(self) -> None:
        backups = self.backups()
        if not backups:
            log("No backups found.", Fore.RED)
            return
        print("\nBackups:")
        for i, b in enumerate(backups, 1):
            print(f"  {i}) {b}")
        try:
            n = int(input("\nSelect (0 to cancel): ").strip())
            if n == 0 or n > len(backups):
                return
            zip_path = os.path.join(self._output, backups[n - 1])
        except ValueError:
            log("Invalid selection.", Fore.RED)
            return
        if input("Type 'yes' to confirm: ").strip().lower() != "yes":
            return
        self._restore(zip_path)

    def run(self) -> None:
        log("Initial backup...")
        self._do_backup()

        observer = Observer()
        observer.schedule(SaveHandler(self), self._config.base_save_folder, recursive=False)
        observer.start()
        log("Watching for changes, press 'b' to backup, Ctrl+C to quit...")

        old_settings = termios.tcgetattr(sys.stdin)
        try:
            tty.setcbreak(sys.stdin.fileno())
            while self._running:
                if sys.stdin in select.select([sys.stdin], [], [], 0.1)[0]:
                    ch = sys.stdin.read(1)
                    if ch.lower() == 'b':
                        log("Manual backup triggered...", Fore.CYAN)
                        self._do_backup()
        except KeyboardInterrupt:
            pass
        finally:
            termios.tcsetattr(sys.stdin, termios.TCSADRAIN, old_settings)

        observer.stop()
        observer.join()
        print()


class SaveHandler(FileSystemEventHandler):
    def __init__(self, manager: BackupManager) -> None:
        self._m = manager
        self._last = 0.0

    def _handle(self, event) -> None:
        if event.is_directory or not self._m._running:
            return
        if not any(fnmatch.fnmatch(os.path.basename(event.src_path), p) for p in self._m._config.files_to_save):
            return
        now = time.time()
        if now - self._last < self._m._config.debounce_seconds:
            return
        if now - self._last < self._m._config.min_backup_interval:
            return
        log(f"Change detected: {event.src_path}")
        time.sleep(self._m._config.debounce_seconds)
        if time.time() - self._last >= self._m._config.debounce_seconds and self._m._running:
            self._m._do_backup()
            self._last = time.time()

    on_modified = _handle
    on_created = _handle


if __name__ == "__main__":
    while True:
        print("\n1) Restore\n2) Run\n3) Start D2R\n4) Exit")
        choice = input("> ").strip()
        if choice == "1":
            BackupManager(Config()).restore_menu()
        elif choice == "2":
            BackupManager(Config()).run()
        elif choice == "3":
            subprocess.Popen([Config().d2r_exe])
        elif choice == "4":
            break
