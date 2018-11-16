# Boundvariable (ICFP 06 Programming Challenge):

## Installation

To install the interpreter and get going, install rustup and then:

```
rustup toolchain install nightly-2018-11-04
rustup default nightly-2018-11-04

cargo build --release
```

## Usage

The interpreter can be used after building above by passing the program as the first and only argument:

```
./target/release/interpreter codex.umz
```

## Layout

- `materials` contains things from the challenge website, or things found during the challenge. Notably:
  - `codes.txt` contains a list of all of the publciation codes that I find.
  - `codex.umz` is the heart of the challenge.
- `src` contains Rust code for the interpreter.

## Notes

**Spoiler alert: below are my notes on progress through the challenge: avoid reading if you don't want spoliers!**

The challenge begins with implementing an interpreter following the spec `materials/um-spec.txt`.

Running the interpreter on the Codex provided (and entering the decryption key provided of `(\b.bb)(\v.vv)06FHPVboundvarHRAk`) gives the ability to dump some data, which when inspected is seen to contain another program that itself can be run on the interpreter.

This _inner_ program provides a login prompt on running it. It says that one can login as `guest`. At this point I find that my program should not strip newlines, as the actual input required to pass this phase is `'guest\n'`. I'm in!

This is a full blown console that I can navigate around folders and run commands with, awesome!

I think I need to collect the publications, so let's see how many I can find!

This one appears right away (`; ` if used, separates input lines in typed commands):

```code
INTRO.LOG=200@999999|35e6f52e9bc951917c73af391e35e1d
```

Looking in my home folder (`cd /home/guest`) I see `a.out`. If I type `run a.out` (some commands found by typing `help`) a file called `core` appears. `cat core` shows:

```code
INTRO.OUT=5@999999|69ca684f8c787cfe06694cb26f74a95
```

Running `mail; 1` shows a loan email with:

```code
INTRO.MUA=5@999999|b9666432feff66e528a17fb69ae8e9a
```

The `telnet` command is also available. I eventually had a play with it and found that `telnet localhost 80` gives back a response! Sending `GET / HTTP/1.1` gives back some html containing the code:

```code
INTRO.WEB=15@999999|071345f65fa1a6a2410986c7bb0ac85
```

From the first email, I can see that `/bin/qbasic` and `/bin/umodem` are additional commands I can run. the former looks like it compiles qbasicish code and the latter looks like a way to write out files from input. `cd home/guest/code; cat hack.bas` shows some qbasicish code that is clearly naffed up at the end.

Testing `/bin/umodem` with a cleaned up version of `hack.bas` is successful; looks like it'll be easy enough to paste content into the terminal for new files. This gives back a code as well:

```code
INTRO.UMD=10@999999|7005f80d6cd9b7b837802f1e58b11b8
```

Even better, it can crack a couple of passwords. Eventually I extended it (`material/betterhack.bas`) to handle the "complex" cases hinted at in the file, which helped a little more. Here are the passwords I have found using it (2 basic and one complex):

```
howie: xyzzy
ohmega: bidirectional
ftd: falderal90
```

I still need passwords for `knr`, `gardener`, `yang`, `hmonk` and `bbarker` at this point.

I have a little snoop around, and then I begin playing an adventure game I find in `howie`. I wondered how useful this might be but quickly found a code playing (combining slides and bullet-point), which leads me to think I should play more..

```code
ADVTR.CMB=5@999999|764e8a851411c66106e130374d8abbb
```

I also learn that I can use a `switch goggles` command for different output styles. This may come in handy! Incinerating a red pill I picked up gives me:

```code
ADVTR.INC=5@999999|f95731ab88952dfa4cb326fb99c085f
```

Combining items together to make a keypad (slightly tricky; don't just combine everything you can!) and using the keypad one step south gives:

```code
ADVTR.KEY=20@999999|36995486a5be3bd747d778916846d2d
```

A transcript of the commands needed after to build a keypad to use in the first room for my own benefit (this is basically my save game):

```
go north
take bolt
take spring
incinerate spring
take button
take processor
take red pill
incinerate red pill
take radio
take cache
take blue transistor
combine radio transistor
take antenna
incinerate antenna
take screw
combine processor cache
take motherboard
combine motherboard screw
take A-1920-IXB
combine A-1920-IXB radio
combine A-1920-IXB processor
combine A-1920-IXB bolt
take red transistor
take keypad
combine A-1920-IXB transistor
combine A-1920-IXB motherboard
combine keypad button
combine keypad motherboard
go south
use keypad
```

Escaping the room and going west a couple of times we find an `/etc/passwd`. examining that gives more passwords:

```
howie:xyzzy:Howard Curry:/home/howie
yang:U+262F:Y Yang:/home/yang
hmonk:COMEFROM:Harmonious Monk:/home/hmonk
```

I still need passwords for `knr`, `gardener` and `bbarker` at this point.

Looking around more, I can see that a _downloader_ and _uploader_ need assembling here. There are many, _many_ pieces scattered in nearby tiles which work together to help assemble these. I think that the purpose of being able to switch goggles is to make this information easier to copy out and injest into a program all at once to solve. I wonder about just writing a thing which connects to the input/output and runs the adventure game to completion for me.

Taking a break, I finish off the `betterhack.bas` file which gives access to `ftd`'s account. here I find `icfp.exe`, which can be passed codes and gives back a score and login credentials; it looks like as my score increases I might obtain better login credentials. I also see and run `ml19100.exe` which gives back the code `BASIC.MLC=100@999999|8f8f7b233a9deb154cbcd5314b8e930`, and a `TODO` file which gives access to `hmonk`'s account (I already have it now). It doesn't look like there is anything else of interest in this account.
