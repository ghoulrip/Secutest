param(
  [Parameter(Mandatory = $true)][string]$IssueId,
  [Parameter(Mandatory = $true)][string]$Body,
  [string]$ApiBase = ""
)

if ([string]::IsNullOrWhiteSpace($ApiBase)) {
  $ApiBase = ($env:PAPERCLIP_API_URL -replace '/api$', '') -replace '/$', ''
}

$headers = @{
  Authorization  = "Bearer $env:PAPERCLIP_API_KEY"
  'X-Paperclip-Run-Id' = $env:PAPERCLIP_RUN_ID
}

$payload = @{ body = $Body } | ConvertTo-Json -Depth 10

try {
  $resp = Invoke-RestMethod -Method Post -Uri "$ApiBase/api/issues/$IssueId/comments" -Headers $headers -ContentType 'application/json; charset=utf-8' -Body ([System.Text.Encoding]::UTF8.GetBytes($payload))
  Write-Host "OK comment id: $($resp.id)"
} catch {
  Write-Host "FAILED: $($_.Exception.Message)"
  if ($_.ErrorDetails) { Write-Host $_.ErrorDetails.Message }
  exit 1
}
