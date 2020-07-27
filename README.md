# Eco2-rust
Take control of your Danfoss Eco™ 2 thermostats!

Not at all done by now. It can connect to a thermostat and show its name,
various temperatures (set-point temperatures and the room's current temperature),
and schedules.

Basically, this is a Rust version of my previous
[C#/Xamarin version](https://github.com/olefriis/Eco2). With Rust and the
[btleplug library](https://github.com/deviceplug/btleplug), we get Linux and
Windows support, in addition to Mac support. (I haven't actually tried this
out, so any feedback and pull requests are welcome.)

## Why?
There are many more or less smart thermostats on the market at various price
points. Most of these contain some kind of central hub which connects to an
online service and lets you control your home while you are away, lets you group
your thermostats and easily schedule whole rooms at a time.

And then there's the Danfoss Eco™ 2.

The Eco 2 is a very cheap thermostat based on Bluetooth LE. It does not support
any kind of central hub, but instead relies on you using the associated app to
connect to all of your thermostats in turn and set up individual schedules. This
is very time-consuming.

This project is intended as an open-source "reference implementation" of how to
interface with the Eco 2 peripherals. Out of the box, it will hopefully turn
into something that will help you quickly set up your thermostats, set vacation
mode on and off, read battery levels, etc. It can also work as the foundation
for creating the missing piece in the Eco 2 ecosystem - a hub.

## Is this in any way an official project?
No. And if you brick your thermostats while using this tool, tough luck.

## Limitations
Currently only reads from the thermostats. I'll add support for writing back to
the thermostats at some point.

PIN codes are not supported.

In general this project is not aiming at getting full feature parity with the
official apps. I think it's OK if you need to use the app to enable or disable
adaptive learning, to switch between horizontal and vertical installation, and
other things that you will probably only do once when setting up the thermostat.

## Usage
You can run all commands through `cargo run`, or you can do `cargo build` once,
put the resulting `target/debug/eco2` command on your path, and just run `eco2`.
Below, we assume the latter.

For now, only the `scan`, `read`, and `show` commands are implemented.

### Scanning for nearby thermostats
Run `eco2 scan`, wait 2 minutes, and see which thermostats your computer could
see. For example:

```
$ eco2 scan
Scanning for 2 minutes. Please wait.
0;0:04:2F:C0:F2:58;eTRV - address: 66:66:64:35:37:30
0;0:04:2F:06:24:D1;eTRV - address: 61:61:35:66:61:39
0;0:04:2F:C0:F3:0C;eTRV - address: 33:62:31:63:64:66
0;0:04:2F:06:24:DD;eTRV - address: 35:39:66:66:61:64
```

### Reading from a thermostat
You can now read from any of the thermostats shown by the `scan` command. You do
that by taking one of the values in the first column shown by the `scan`
output - for example `0;0:04:2F:06:24:D1;eTRV` from line 2 in the output above.

If this is the first time you connect to the thermostat, the `scan` command needs
to be able to read a secret key from the thermostat. As you may remember when
setting up your thermostat from the app, you are required to click the timer button
on the thermostat. The app asks you to do this and magically finds out when you have
clicked the button. This tool instead asks you to click the button and subsequently
press enter on your keyboard.

Also, remember to put the thermostat serial number in quotes, or your shell will get
very confused.

Example:

```
$ eco2 read '0;0:04:2F:06:24:D1;eTRV'
Reading from 0;0:04:2F:06:24:D1;eTRV for the first time...
.....Got our peripheral: 0;0:04:2F:06:24:D1;eTRV
This is the first time you connect to this thermostat, so we need to fetch the secret key.
Please click the timer button on the thermostat, then press enter on your keyboard to continue connecting.

Connected to peripheral
Wrote pin code
```

That's it. We have now read all relevant values from the thermostat.

### Showing the values from the thermostat
Of course you want to see what we just read. So you should use the `show` command.

```
$ eco2 show '0;0:04:2F:06:24:D1;eTRV'
Name: Alrum opgang

Set-point/room temperature: 19°C / 23.5°C
Vacation/frost protection temperature: 17°C / 6°C

Schedule mode: Scheduled

Daily Schedules
Monday: Away until 05:00 - Home until 20:00 - Away until 24:00
Tuesday: Away until 04:30 - Home until 24:00
Wednesday: Away until 08:00 - Home until 10:00 - Away until 14:00 - Home until 17:30 - Away until 24:00
Thursday: Away until 05:30 - Home until 07:30 - Away until 09:30 - Home until 12:00 - Away until 14:30 - Home until 18:00 - Away until 24:00
Friday: Away until 24:00
Saturday: Home until 24:00
Sunday: Home until 03:30 - Away until 20:30 - Home until 24:00
```

### Details
All the values read from thermostats are stored in the `.thermostats.json` file
in your home directory. If you somehow end up in a weird state, just delete
that file and start over.

## So... Did you hack the Eco 2 security?
If you've read the official specification for the Eco 2, you may have noticed
that it mentions that data on the device is secure, and that the security has
been audited by external parties. So did we break the security to make this
tool?

Short answer: No.

Without physical access to the thermostat, all you can do is connect to it and
read the battery level, the model number, firmware version, and other harmless
data. If you set a PIN code on the thermostat (in the app), you even need to
know that PIN code in order to retrieve this data.

To enable the tool to read the settings on the device, upon first connection you
need to physically push the "timer" button on the thermostat. This will, for a
short while, reveal an encryption key that can be used to read the remaining
data on the thermostat.

In other words, the Eco 2 security is sensible and seems well-implemented, and
in order to access the data, this tool (presumably!) does exactly the same as
the iOS and Android apps do.

## Thank you!
This tool wouldn't have been possible without the
[btleplug library](https://github.com/deviceplug/btleplug) by @qdot.

The XXTEA implementation is from the [XXTEA-Rust](https://github.com/Hanaasagi/XXTEA-Rust)
project, modified such that the length of the input byte array is not included
in the encoded data. Accompanying license file is included in this repository.
