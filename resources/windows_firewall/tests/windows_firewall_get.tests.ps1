# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Windows Firewall get tests' -Skip:(!$IsWindows) {
    BeforeAll {
        $resourceType = 'Microsoft.Windows/Firewall'
    }

    Context 'Get by name' {
        It 'Returns rule info for an existing firewall rule' {
            $builtInRule = Get-NetFirewallRule | Select-Object -First 1
            $json = @{ name = $builtInRule.Name } | ConvertTo-Json -Compress
            $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $output = $out | ConvertFrom-Json
            $result = $output.actualState
            $result.name | Should -BeExactly $builtInRule.Name
            $result._exist | Should -BeTrue
            $result.direction | Should -Not -BeNullOrEmpty
            $result.action | Should -Not -BeNullOrEmpty
        }

        It 'Returns _exist false for a non-existent rule' {
            $json = @{ name = 'DSC_NonExistent_FirewallRule_12345' } | ConvertTo-Json -Compress
            $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $output = $out | ConvertFrom-Json
            $result = $output.actualState
            $result.name | Should -BeExactly 'DSC_NonExistent_FirewallRule_12345'
            $result._exist | Should -BeFalse
            $result.PSObject.Properties.Name | Should -Not -Contain 'direction'
        }
    }

    Context 'Rule properties' {
        It 'Returns valid direction values' {
            $builtInRule = Get-NetFirewallRule | Select-Object -First 1
            $json = @{ name = $builtInRule.Name } | ConvertTo-Json -Compress
            $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
            $output = $out | ConvertFrom-Json
            $result = $output.actualState
            $result.direction | Should -BeIn @('Inbound', 'Outbound')
        }

        It 'Returns valid action values' {
            $builtInRule = Get-NetFirewallRule | Select-Object -First 1
            $json = @{ name = $builtInRule.Name } | ConvertTo-Json -Compress
            $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
            $output = $out | ConvertFrom-Json
            $result = $output.actualState
            $result.action | Should -BeIn @('Allow', 'Block')
        }

        It 'Returns enabled as a boolean' {
            $builtInRule = Get-NetFirewallRule | Select-Object -First 1
            $json = @{ name = $builtInRule.Name } | ConvertTo-Json -Compress
            $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
            $output = $out | ConvertFrom-Json
            $result = $output.actualState
            $result.enabled | Should -BeOfType [bool]
        }

        It 'Returns profiles as an array' {
            $builtInRule = Get-NetFirewallRule | Select-Object -First 1
            $json = @{ name = $builtInRule.Name } | ConvertTo-Json -Compress
            $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
            $output = $out | ConvertFrom-Json
            $result = $output.actualState
            $result.profiles | Should -Not -BeNullOrEmpty
            $validProfiles = @('Domain', 'Private', 'Public')
            foreach ($profile in $result.profiles) {
                $profile | Should -BeIn $validProfiles
            }
        }
    }
}
