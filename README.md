**This program is no longer maintained.**

As Killjoy is no longer maintained, this killjoy notifier will also not be updated anymore,
not even to fix security vulnerabilities.

killjoy Notifier: Slack
==============================

Generate slack notifications on behalf of
[killjoy](https://github.com/Ichimonji10/killjoy).

killjoy is a systemd unit monitoring application. It discovers systemd units and
tracks their states. When a unit changes to a state of interest, killjoy
contacts notifiers. This application is a notifier which, upon being contacted
by killjoy, will post a [Slack](https://slack.com/) message using a [webhook](https://api.slack.com/messaging/webhooks).

The code for this notifier was based in part on 
https://github.com/Ichimonji10/killjoy-notifier-notification.

Concepts
--------

First, read the concepts section in the
[killjoy](https://github.com/Ichimonji10/killjoy) documentation.

When properly installed, this application will be auto-started whenever 
a D-Bus message is sent to it. When started, this application will consume
 all messages (presumably from killjoy) in its message queue, and then idle.

Installation
------------

Rust developers may install this app from source. Note that libdbus must be
installed. (On Ubuntu, this is provided by the `libdbus-1-dev` package.)

```bash
git clone https://github.com/kennep/killjoy-notifier-slack.git
cd killjoy-notifier-notification
cargo build
```

Note: no systemd unit scripts or installation scripts are included.
The scripts from e.g. https://github.com/Ichimonji10/killjoy-notifier-notification
can be used as a starting point.

Configuration
-------------

This notifier expects a configuration file called `slack-notifier.json` to 
be installed in the same directory as the main killjoy configuration file.

Here is an example configuration file with all configuration keys populated:

```json
{
    "webhook_url": "https://hooks.slack.com/services/YOUR_UNIQUE_WEBOOOK_URL",
    "username": "My user",
    "channel": "Channel name",
    "icon_emoji": ":robot_face:"
}
```

The `webhook_url` key is required, and contains your Slack webhook URL.
The `username`, `channel` and `icon_emoji` keys are optional. If present,
they specify the username to post as, the channel to post in and the emoji
to use.

If `username` or `channel` are not present, the defaults for the Slack 
webhook URL is used. If `icon_emoji` is not present, then `:robot_face:` is
used as the emoji.

Usage
-----

Define a notifier in killjoy's configuration file:

```json
"slack": {
    "bus_type": "session",
    "bus_name": "com.wangpedersen.KilljoyNotifierSlack1"
}
```

Then, list it in a rule's list of `notifiers`.

When this application receives a message, it will generate a slack message.

Changelog
---------

See annotated git tags.

License
-------

This application is licensed under the GPLv3 or any later version.
