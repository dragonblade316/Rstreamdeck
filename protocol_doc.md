# Protocol Documentation

This is the documentation to the plugin protocol. 

You probably should not use this, instead use RstreamdeckPluginLib (prefered) or Rstreamdeck-lib (if you want to make your own rust control library or something).

However, if you want to port the api to another language, this doc will be needed.
With that disclaimer out of the way lets begin.

When Rstreamdeck is launched by the user a few things will happen. Among those tasks is starting all plugins in the /.config/rstreamdeck/plugins dir. 
Being loaded from the plugins dir is the only way to successfully connect to the plugin api since the app will only accept a connection when it is expecting it.

Now that we are connected it is important we understand the structure of a message. 
The first part of a message is a u64 length spesifier (Could change since I dont know why I made it this long). After this you will send a json encoded string of the spesified length.
This Json will be interpreted by Serde Json and converted to and enum (likely containing a struct) which will be used within the application.

Once your application is started you need to connect to the UnixSocket at ""/tmp/rdeck.sock". After this your plugin will send its initial report.
Here is the structure of the initial report:
(example pending finalization of the protocol)

