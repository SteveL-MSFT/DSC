# Creating web visualizer

## Goals

Create web visualizer in the `extensions/web-visualizer` directory.  The visualizer will allow users to graphically load a DSC configuration JSON file, edit it,
execute the configuration, and save the file.

## Interaction with DSC

The visualizer will create a long running process using `dsc mcp` and use JSON-RPC over STDIO to communicate with the process.
The JSON-RPC messages are defined in `dsc/src/mcp` folder.
Upon exit of the visualizer, the `dsc` process should be terminated.

## Layout

The layout of the visualizer should be responsive and adjust to different screen sizes.

The layout should be composed of the following components:

+-------------------------------------------------------------------------------+
| Title area                                                                    |
+-------------------------------------------------------------------------------+
| Tool bar                                                                      |
+----------------------+------------------------------+-------------------------+
| Tree view            | Main area (visualization)    | Property editor         |
|                      |                              |                         |
|                      |                              |                         |
|                      |                              |                         |
|                      |                              |                         |
|                      |                              |                         |
|                      |                              |                         |
|                      |                              |                         |
+----------------------+------------------------------+-------------------------+
| Console output area (with tabs)                                               |
+-------------------------------------------------------------------------------+
| Status area                                                                   |
+-------------------------------------------------------------------------------+

1. The title area displays `DSC Visualizer` followed by the version number of the visualizer, then the DSC version as `DSC vX.Y.Z` where X.Y.Z is the version of DSC that the visualizer is using.
2. Tool bar with buttons for loading configuration, saving configuration, executing the configuration, showing/hiding the tree view, showing/hiding the property editor, and showing/hiding the console output area.
3. A tree view to display the structure of the configuration file on the left side of the window, resizable by the user, and should be collapsible to allow more space for the main area.
4. A property editor on the right side of the window to edit the properties of the selected node in the tree view, resizable by the user, and should be collapsible to allow more space for the main area.
5. A console output area at the bottom of the window to display the output of the executed configuration, resizable by the user, and should be collapsible to allow more space for the main area.

- The console output area should have tabs for the standard output, trace logging, and error output.

6. The main area shows a visual representation of the configuration using boxes to represent the resources with lines with arrows to represent the dependencies between the resources.

- Look in the `dsc/examples` folder for sample configuration files to use for testing the visualizer.
- The resource box should display the resource type and name.
- Clicking on a resource box should select the corresponding node in the tree view and display its properties in the property editor.

7. The status area at the bottom of the window should display the current status of the visualizer, such as "Ready", "Loading configuration...", "Executing configuration...", etc. along with the name of the loaded configuration file if any.

### Main Area

The main area should show a visual representation of the configuration using boxes to represent the resources with lines with arrows to represent the dependencies between the resources.

The `executionInformation` of the configuration file should not be displayed in the main area.
The `directives` of the configuration file should be displayed in the main area and if selected, should show the properties of the directives in the property editor.

### Property editor

Use the `show_dsc_schema` specifying `type` as `resource` to get the JSON schema for the DSC resource within the configuration file.

The property editor should show and allow the user to edit the corresponding properties of the selected node in the tree or the main area.
The editor displays the name and type of the resource node, and a list of its properties with their current values. The user can edit the values of the properties and save the changes back to the configuration file.

Use the `show_dsc_resource` JSON RPC method to get the JSON schema for the selected resource and render the appropriate input fields for each property based on its type (e.g., string, integer, boolean, array, etc.). The editor should also support nested properties and allow the user to expand/collapse them as needed.  Enumerations should be rendered as dropdowns, booleans as checkboxes, and arrays should allow adding/removing items.

If the selected node is `directives`, the property editor should show the properties of the directives and allow the user to edit them.  Use the `show_dsc_schema` JSON RPC method specifying `type` as `configuration` to get the JSON schema for the configuration, but use the `directives` property of the schema to render the properties of the directives in the property editor.

## Themes

The visualizer should support built-in themes initially light and dark mode.

## Accessibility

The visualizer should be accessible and support keyboard navigation and screen readers meeting WCAG 2.1 AA standards.
