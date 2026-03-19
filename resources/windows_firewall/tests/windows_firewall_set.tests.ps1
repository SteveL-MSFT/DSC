# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

Describe 'Windows Firewall set tests' -Skip:(!$IsWindows) {
    BeforeDiscovery {
        $isAdmin = if ($IsWindows) {
            $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
            $principal = [Security.Principal.WindowsPrincipal]$identity
            $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
        }
        else {
            $false
        }
    }

    BeforeAll {
        $resourceType = 'Microsoft.Windows/Firewall'
        $testRuleName = 'DSC_Test_FirewallRule'

        function Get-RuleState {
            param([string]$Name)
            $json = @{ name = $Name } | ConvertTo-Json -Compress
            $out = $json | dsc resource get -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            return ($out | ConvertFrom-Json).actualState
        }
    }

    Context 'Create a new rule' -Skip:(!$isAdmin) {
        AfterEach {
            Remove-NetFirewallRule -Name $testRuleName -ErrorAction SilentlyContinue
        }

        It 'Creates a new inbound block rule' {
            $json = @{
                name      = $testRuleName
                direction = 'Inbound'
                action    = 'Block'
                protocol  = 'TCP'
                localPorts = '8080'
                enabled   = $true
            } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result = ($out | ConvertFrom-Json).afterState
            $result._exist | Should -BeTrue
            $result.name | Should -BeExactly $testRuleName
            $result.direction | Should -BeExactly 'Inbound'
            $result.action | Should -BeExactly 'Block'
            $result.enabled | Should -BeTrue
        }

        It 'Creates a new outbound allow rule' {
            $json = @{
                name      = $testRuleName
                direction = 'Outbound'
                action    = 'Allow'
                protocol  = 'UDP'
                remotePorts = '53'
            } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result = ($out | ConvertFrom-Json).afterState
            $result._exist | Should -BeTrue
            $result.direction | Should -BeExactly 'Outbound'
            $result.action | Should -BeExactly 'Allow'
        }

        It 'Creates a rule with specific profiles' {
            $json = @{
                name      = $testRuleName
                direction = 'Inbound'
                action    = 'Allow'
                profiles  = @('Domain', 'Private')
            } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result = ($out | ConvertFrom-Json).afterState
            $result.profiles | Should -Contain 'Domain'
            $result.profiles | Should -Contain 'Private'
            $result.profiles | Should -Not -Contain 'Public'
        }
    }

    Context 'Delete a rule' -Skip:(!$isAdmin) {
        It 'Removes a firewall rule when _exist is false' {
            New-NetFirewallRule -Name $testRuleName -DisplayName $testRuleName -Direction Inbound -Action Allow | Out-Null
            $json = @{
                name   = $testRuleName
                _exist = $false
            } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result = ($out | ConvertFrom-Json).afterState
            $result._exist | Should -BeFalse
            Get-NetFirewallRule -Name $testRuleName -ErrorAction SilentlyContinue | Should -BeNullOrEmpty
        }

        It 'Succeeds when removing a non-existent rule' {
            $json = @{
                name   = $testRuleName
                _exist = $false
            } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result = ($out | ConvertFrom-Json).afterState
            $result._exist | Should -BeFalse
        }
    }

    Context 'Modify an existing rule' -Skip:(!$isAdmin) {
        BeforeEach {
            New-NetFirewallRule -Name $testRuleName -DisplayName $testRuleName `
                -Direction Inbound -Action Allow -Protocol TCP -LocalPort 9090 `
                -Enabled True | Out-Null
        }

        AfterEach {
            Remove-NetFirewallRule -Name $testRuleName -ErrorAction SilentlyContinue
        }

        It 'Can change the action from Allow to Block' {
            $json = @{
                name   = $testRuleName
                action = 'Block'
            } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result = ($out | ConvertFrom-Json).afterState
            $result.action | Should -BeExactly 'Block'
        }

        It 'Can disable a rule' {
            $json = @{
                name    = $testRuleName
                enabled = $false
            } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result = ($out | ConvertFrom-Json).afterState
            $result.enabled | Should -BeFalse
        }

        It 'Returns full rule state after set' {
            $json = @{ name = $testRuleName; action = 'Block' } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result = ($out | ConvertFrom-Json).afterState
            $result.name | Should -Not -BeNullOrEmpty
            $result._exist | Should -BeTrue
            $result.direction | Should -Not -BeNullOrEmpty
            $result.action | Should -Not -BeNullOrEmpty
            $result.enabled | Should -Not -BeNullOrEmpty
        }
    }

    Context 'Input validation' -Skip:(!$isAdmin) {
        It 'Fails when name is not provided' {
            $json = @{ action = 'Block' } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>&1
            $LASTEXITCODE | Should -Not -Be 0
        }

        It 'Fails when input JSON is invalid' {
            $out = 'not-json' | dsc resource set -r $resourceType -f - 2>&1
            $LASTEXITCODE | Should -Not -Be 0
        }
    }

    Context 'Idempotent set' -Skip:(!$isAdmin) {
        BeforeAll {
            New-NetFirewallRule -Name $testRuleName -DisplayName $testRuleName `
                -Direction Inbound -Action Allow -Enabled True | Out-Null
        }

        AfterAll {
            Remove-NetFirewallRule -Name $testRuleName -ErrorAction SilentlyContinue
        }

        It 'Succeeds when only name is provided (no properties to change)' {
            $json = @{ name = $testRuleName } | ConvertTo-Json -Compress
            $out = $json | dsc resource set -r $resourceType -f - 2>$testdrive/error.log
            $LASTEXITCODE | Should -Be 0 -Because (Get-Content -Raw $testdrive/error.log)
            $result = ($out | ConvertFrom-Json).afterState
            $result.name | Should -BeExactly $testRuleName
            $result._exist | Should -BeTrue
        }
    }
}
