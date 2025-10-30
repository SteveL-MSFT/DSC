# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

[DscResource()]
class TestPSResource {
    [DscProperty(Key)]
    [string] $Name

    [DscProperty()]
    [string] $PowerShellVersion

    [TestPSResource] Get() {
        $result = [TestPSResource]::new()
        $result.Name = 'Example'
        $result.PowerShellVersion = $global:PSVersionTable.PSVersion.ToString()
        return $result
    }

    [void] Set() {
        # No-op
    }

    [bool] Test() {
        return $true
    }

    static [TestPSResource[]]Export() {
        return @([TestPSResource]@{
            Name = "Example"
            PowerShellVersion = $global:PSVersionTable.PSVersion.ToString()
        })
    }
}
