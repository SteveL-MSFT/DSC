# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

@{
    RootModule = 'adapted.psm1'
    ModuleVersion = '0.0.1'
    GUID = '6fa1d279-6936-4111-830f-b634d7eb4e27'
    Author = 'Microsoft'
    CompanyName = 'Microsoft Corporation'
    Copyright = '(c) Microsoft. All rights reserved.'
    FunctionsToExport = @()
    CmdletsToExport = @()
    VariablesToExport = @()
    AliasesToExport = @()
    DscResourcesToExport = @('TestPSResource')
    PrivateData = @{
        PSData = @{
            DscCapabilities = @(
                'get'
                'export'
            )
        }
    }
}
