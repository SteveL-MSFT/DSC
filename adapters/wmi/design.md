# New WMI Adapter

All folder paths in this doc are from the repo root.

Create a new WMI adapter for DSC.  This should be written in rust and put in the `./adapters/wmi` folder.  Leave the existing files
in there.

## Design goals

Enable use of adapted resource manifest to have nested content using the `content` property that creates
a bi-directional mapping between JSON and WMI.

See `./resources/windows_peronalization/windows_personalization.dsc.adaptedResource.yaml` as a similar example which
creates a bi-directional mapping between JSON and the Windows registry.  The registry adapter code in
`./resources/registry/src/adapter.rs` is an example of how the mapping works.

Unlike the existing WMI adapter, there is no goal to allow arbitrary access to WMI classes vis DSC for this adapter at this time.
That capability will be added later, so consider that aspect as part of the design, but not implemented other than TODO comments.

### Get

For a get operation, internally the WMI adapter should create a WQL query with the input provided transformed
into the appropriate WMI properties and values for a query.  The WQL query should be emitted as a DSC debug trace.

### Set

Set should use WMI CreateOrUpdate.  This means if the user does not exist, it gets created.

## Adapted Resource example

The file `./resources/windows_useraccount/windows_useraccount.dsc.adaptedResource.yaml` is an example adapted resource
manifest that should be supported by this WMI adapter.

In this case, the adapted resource leverages the win32_useraccount class in the root/cimv2 namespace.
The properties listed under `content` that do not have a `mapJsonToWmi` property are directly mapped.
This means the JSON property value which is a key value in the map becomes the WMI property value.
Since all key values are strings, there would need to be a helper function that converts the key string
value to the appropriate JSON type and uses that for the WMI value.

## Code Design

## Tests
