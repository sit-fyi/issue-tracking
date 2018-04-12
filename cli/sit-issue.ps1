$scriptPath = split-path -parent $MyInvocation.MyCommand.Definition
$cli = Get-Content -Raw -Path "$scriptPath\sit-issue.yaml"
$output = ($cli | & $Env:SIT args -- "sit issue" $args) -split '\n'
$command = $output[0]
$matched = $output[1..($output.Length - 1)]
switch -Regex ($command) {
  '^list'
  {
    $query = & $Env:SIT config -q "'issue-tracking'.cli.oneliner || 'join(\' | \', [id, summary])'" repository
    & $Env:SIT items -q $query
  }
  '^show' {
    switch -Regex ($matched) {
      '^ID\s+\d+\s+\d+\s(.+)' {
        $ID = Invoke-Expression $Matches[1]
        Write-Host -NoNewline "Summary: "
        & $Env:SIT reduce -q summary $ID
        Write-Host Details:
        Write-Host
        & $Env:SIT reduce -q details $ID
        Write-Host
        Write-Host Created at
        & $Env:SIT reduce -q timestamp $ID
        Write-Host by
        & $Env:SIT reduce -q authors $ID
      }
    }
  }
  default {
    $cli | & $Env:SIT args --help
  }
}
