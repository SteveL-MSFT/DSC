# Creating web visualizer

## Goals

Create web visualizer in the `extensions/web-visualizer` directory.  The visualizer will allow users to graphically load a DSC configuration JSON file, edit it,
execute the configuration, and save the file.

## Interaction with DSC

The visualizer will create a long running process using `dsc mcp` and use JSON-RPC over STDIO to communicate with the process.
The JSON-RPC messages are defined in `dsc/src/mcp` folder.
Upon exit of the visualizer, the `dsc` process should be terminated.

## GUI

The GUI should be composed of the following components:

1. Tool bar with buttons for loading, saving, and executing the configuration.
2. A tree view to display the structure of the configuration file on the left side of the window.
3. A property editor on the right side of the window to edit the properties of the selected node in the tree view.
4. A console output area at the bottom of the window to display the output of the executed configuration.

- The console output area should have tabs for the standard output, trace logging, and error output.

5. The main area shows a visual representation of the configuration using boxes to represent the resources with lines with arrows to represent the dependencies between the resources.

- Look in the `dsc/examples` folder for sample configuration files to use for testing the visualizer.
- The resource box should display the resource type and name.
- Clicking on a resource box should select the corresponding node in the tree view and display its properties in the property editor.

## Property editor

The property editor should show and allow the user to edit the corresponding

