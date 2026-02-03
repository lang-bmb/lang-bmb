$bmb_exe = 'C:/Users/iyu-nb02/AppData/Local/Temp/sorting_test'
$c_exe = 'C:/Users/iyu-nb02/AppData/Local/Temp/sorting_c_v2.exe'

Write-Host '=== BMB (while loop version) ===' -ForegroundColor Green
$bmb_times = @()
for ($i = 1; $i -le 5; $i++) {
    $sw = [Diagnostics.Stopwatch]::StartNew()
    & $bmb_exe | Out-Null
    $sw.Stop()
    $bmb_times += $sw.ElapsedMilliseconds
}
Write-Host ('Times (ms): ' + ($bmb_times -join ', '))
Write-Host ('Average: ' + [math]::Round(($bmb_times | Measure-Object -Average).Average, 2) + 'ms')

Write-Host ''
Write-Host '=== C baseline ===' -ForegroundColor Green
$c_times = @()
for ($i = 1; $i -le 5; $i++) {
    $sw = [Diagnostics.Stopwatch]::StartNew()
    & $c_exe | Out-Null
    $sw.Stop()
    $c_times += $sw.ElapsedMilliseconds
}
Write-Host ('Times (ms): ' + ($c_times -join ', '))
Write-Host ('Average: ' + [math]::Round(($c_times | Measure-Object -Average).Average, 2) + 'ms')

$ratio = ($bmb_times | Measure-Object -Average).Average / ($c_times | Measure-Object -Average).Average
Write-Host ''
Write-Host ('BMB/C ratio: ' + [math]::Round($ratio, 2) + 'x')
