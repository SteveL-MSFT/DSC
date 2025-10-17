$packages = Get-AppxPackage
$extensions = @(
    '*.dsc.adaptedresource.json'
    '*.dsc.adaptedresource.yaml'
    '*.dsc.adaptedresource.yml'
    '*.dsc.manifests.json'
    '*.dsc.manifests.yaml'
    '*.dsc.manifests.yml'
    '*.dsc.resource.json'
    '*.dsc.resource.yaml'
    '*.dsc.resource.yml'
)
foreach ($package in $packages) {
    $manifests = Get-ChildItem -Path "$($package.InstallLocation)\*" -File -Include $extensions -ErrorAction Ignore
    foreach ($manifest in $manifests) {
        @{ manifestPath = $manifest.FullName } | ConvertTo-Json -Compress
    }
}
