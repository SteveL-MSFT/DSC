# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Windows Firewall export tests' -Skip:(!$IsWindows) {
    BeforeAll {
        $resourceType = 'Microsoft.Windows/Firewall'

        function Invoke-DscExport {
            param(
                [string]$InputJson
            )
            if ($InputJson) {
                $raw = $InputJson | dsc resource export -r $resourceType -f - 2>$testdrive/error.log
            }
            else {
                $raw = dsc resource export -r $resourceType 2>$testdrive/error.log
            }
            $parsed = $raw | ConvertFrom-Json
            return $parsed
        }
    }

    Context 'Export without filter' {
        It 'Returns multiple rules' {
            $result = Invoke-DscExport
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result.resources.Count | Should -BeGreaterThan 10
        }

        It 'Each exported rule has required properties' {
            $result = Invoke-DscExport
            foreach ($resource in $result.resources | Select-Object -First 5) {
                $rule = $resource.properties
                $rule.name | Should -Not -BeNullOrEmpty
                $rule._exist | Should -BeTrue
                $rule.direction | Should -BeIn @('Inbound', 'Outbound')
                $rule.action | Should -BeIn @('Allow', 'Block')
                $rule.enabled | Should -BeOfType [bool]
            }
        }

        It 'Sets the correct resource type on each entry' {
            $result = Invoke-DscExport
            foreach ($resource in $result.resources | Select-Object -First 5) {
                $resource.type | Should -BeExactly $resourceType
            }
        }
    }

    Context 'Export with name filter' {
        It 'Filters by exact rule name' {
            $knownRule = Get-NetFirewallRule | Select-Object -First 1
            $json = @{ name = $knownRule.Name } | ConvertTo-Json -Compress
            $result = Invoke-DscExport -InputJson $json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result.resources.Count | Should -Be 1
            $result.resources[0].properties.name | Should -BeExactly $knownRule.Name
        }

        It 'Filters by name with wildcard' {
            $json = @{ name = '*Remote*' } | ConvertTo-Json -Compress
            $result = Invoke-DscExport -InputJson $json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result.resources.Count | Should -BeGreaterThan 0
            foreach ($resource in $result.resources) {
                $resource.properties.name | Should -BeLike '*Remote*'
            }
        }

        It 'Returns empty when name filter matches nothing' {
            $json = @{ name = 'nonexistent_rule_xyz_12345' } | ConvertTo-Json -Compress
            $result = Invoke-DscExport -InputJson $json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result.resources.Count | Should -Be 0
        }
    }

    Context 'Export with direction filter' {
        It 'Filters by Inbound direction' {
            $json = @{ direction = 'Inbound' } | ConvertTo-Json -Compress
            $result = Invoke-DscExport -InputJson $json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result.resources.Count | Should -BeGreaterThan 0
            foreach ($resource in $result.resources) {
                $resource.properties.direction | Should -BeExactly 'Inbound'
            }
        }

        It 'Filters by Outbound direction' {
            $json = @{ direction = 'Outbound' } | ConvertTo-Json -Compress
            $result = Invoke-DscExport -InputJson $json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result.resources.Count | Should -BeGreaterThan 0
            foreach ($resource in $result.resources) {
                $resource.properties.direction | Should -BeExactly 'Outbound'
            }
        }
    }

    Context 'Export with action filter' {
        It 'Filters by Allow action' {
            $json = @{ action = 'Allow' } | ConvertTo-Json -Compress
            $result = Invoke-DscExport -InputJson $json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result.resources.Count | Should -BeGreaterThan 0
            foreach ($resource in $result.resources) {
                $resource.properties.action | Should -BeExactly 'Allow'
            }
        }

        It 'Filters by Block action' {
            $json = @{ action = 'Block' } | ConvertTo-Json -Compress
            $result = Invoke-DscExport -InputJson $json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result.resources.Count | Should -BeGreaterOrEqual 0
            foreach ($resource in $result.resources) {
                $resource.properties.action | Should -BeExactly 'Block'
            }
        }
    }

    Context 'Export with multi-field filter' {
        It 'Filters by direction AND action together' {
            $json = @{ direction = 'Inbound'; action = 'Allow' } | ConvertTo-Json -Compress
            $result = Invoke-DscExport -InputJson $json
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result.resources.Count | Should -BeGreaterThan 0
            foreach ($resource in $result.resources) {
                $resource.properties.direction | Should -BeExactly 'Inbound'
                $resource.properties.action | Should -BeExactly 'Allow'
            }
        }
    }
}
