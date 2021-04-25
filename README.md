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

## Building
You gotta [install Rust](https://www.rust-lang.org/tools/install) first. Then
you can build the tool in release mode with `cargo build --release`, put
`target/release/eco2` on your path, and then just call e.g. `eco2 scan`.

If you are lazy, you can also just build in debug mode and run in one fell
swoop by running e.g. `cargo run scan` on your command-line.

Run the unit tests with `cargo test -- --test-threads=1`. The `-- --test-threads=1`
is necessary because a couple of the tests write to and read from a file on
disk, so they are flaky when run in parallel. (If you know how to set this up
in `Cargo.toml`, please send me a PR...)

## Usage

### Scanning for nearby thermostats
Run `eco2 scan`, wait 2 minutes, and see which thermostats your computer could
see. For example:

```
$ eco2 scan
Scanning for 2 minutes. Please wait.
0:04:2F:C0:F2:58
0:04:2F:06:24:D1
0:04:2F:C0:F3:0C
0:04:2F:06:24:DD
```

### Reading from a thermostat
You can now read from any of the thermostats shown by the `scan` command. You do
that by taking one of the values shown by the `scan` output - for example
`0:04:2F:06:24:D1` the output above. (In fact, this corresponds to the "MAC Address"
that you can see in the Eco 2 app by choosing your thermostat, going to Settings,
and choosing System Information.)

If this is the first time you connect to the thermostat, the `read` command needs
to be able to read a secret key from the thermostat. As you may remember when
setting up your thermostat from the app, you are required to click the timer button
on the thermostat. The app asks you to do this and magically finds out when you have
clicked the button. This tool instead asks you to click the button and subsequently
press enter on your keyboard.

Example:

```
$ eco2 read 0:04:2F:06:24:D1
Reading from 0:04:2F:06:24:D1 for the first time...
.....Found thermostat
This is the first time you connect to this thermostat, so we need to fetch the secret key.
Please click the timer button on the thermostat, then press enter on your keyboard to continue connecting.

Connected to peripheral
Wrote pin code
```

That's it. We have now read all relevant values from the thermostat.

### Showing the values from the thermostat
Of course you want to see what we just read. So you should use the `show` command.

```
$ eco2 show 0:04:2F:06:24:D1
Name: Alrum opgang
74% battery

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

### Updating properties
You can update the set-point temperature and vacation period. You do that by
using two different commands: `eco2 set` and `eco2 sync`.

To update the set-point temperature, run the `eco2 set` command with the serial,
`set-point-temperature`, and the desired temperature in degrees Celcius.

```
$ eco2 set 0:04:2F:06:24:D1 set-point-temperature 21.5
```

This runs really fast, because it only updates the temperature in the tool's
database, which means it will _not_ write to the Eco2 thermostat.

To set the vacation period, use the `vacation-period` parameter instead and specify
the start and end times:

```
$ eco2 set 0:04:2F:06:24:D1 vacation-period "2021-04-05 13:00" "2022-05-12 10:00"
```

Only `00` minutes are accepted. The start and end times are specified in your
computer's time zone.

To clear the vacation period, specify `clear` instead of the dates:

```
$ eco2 set 0:04:2F:06:24:D1 vacation-period clear
```

You can try to run `eco2 show` for your thermostat. It will show the same
values as before, but at the bottom of the output you will now also see:

```
Properties to be written back to thermostat:
Set-point temperature: 21.5
Vacation: 2021-04-05 13:00 - 2022-05-12 10:00
```

In order to write these values back to the thermostat, you will need to run the
`eco2 sync` command:

```
$ eco2 sync 0:04:2F:06:24:D1
..Found thermostat
Connected to peripheral
Wrote pin code
```

This looks very much like the output from the `eco2 read` command you previously
ran. In fact, `eco2 sync` will also read all the values from the thermostat, so
if you haven't set any updated properties on the thermostat, `eco2 read` and
`eco2 sync` will do the same thing. (The initial read from the thermostat will
require an `eco2 read`, though, since this command can take care of reading the
secret from the thermostat, whereas `eco2 sync` will fail if the tool does not
know that secret.)

### Listing thermostats
It's sometimes nice to get an overview of which thermostats you have in your
system. Just call `eco2 list`. It will show the serial, the name, and the
battery percentage for all the thermostats known by the tool.

Please note: This information is based on what was retrieved last time you
called either `eco2 read` or `eco2 sync` for the individual thermostats. The
`eco2 list` command will not connect to any thermostats.

### Deleting/forgetting thermostats
Sometimes you move around thermostats, get rid of some, reset some, etc. To get
rid of a thermostat in the tool, just call e.g. `eco2 forget 0:04:2F:06:24:D1`.

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
