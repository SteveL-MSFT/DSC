targetScope = 'desiredStateConfiguration'

@description('Example of a loop in DSC')
param loopCount int = 3

var loopItems = [for i in range(0, loopCount): i]
resource echo 'Microsoft.DSC.Debug/Echo@2025-06-01' = [for item in loopItems: {
  name: 'example${item}'
  properties: {
    output: 'This is loop item ${item}'
  }
}]
