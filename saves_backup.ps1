$curDate = (Get-Date).ToString('yyyyMMdd_HHmm')
$backupTarget = "D:\Backups\D2R\" + $curDate + "_d2r_savesbackup.7z"

C:\"Program Files"\7-Zip\7z.exe a -t7z $backupTarget C:\Users\tlda\"Saved Games"\"Diablo II Resurrected"\* -r