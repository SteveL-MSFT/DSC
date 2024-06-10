# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Resource get tests' {
    BeforeAll {
        $family = if ($IsWindows) {
            'Windows'
        } elseif ($IsLinux) {
            'Linux'
        } else {
            'macOS'
        }
    }

    It 'Can perform get with name of resource' {
        $out = dsc resource get -r Microsoft/osinfo | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.actualState.family | Should -Be $family
    }


    It 'Can perform get with JSON of the resource' {
        $osinfo = dsc resource list 'Microsoft/osinfo'
        $LASTEXITCODE | Should -Be 0
        $osinfo | Should -Not -BeNullOrEmpty
        $out = dsc resource get -r $osinfo | ConvertFrom-Json
        $LASTEXITCODE | Should -Be 0
        $out.actualState.family | Should -Be $family
    }
}
