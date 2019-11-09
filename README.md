Dofus Generate Stuff
====================

[![Build Status](https://travis-ci.com/remi-dupre/dofus-generate-stuff.svg?branch=master)](https://travis-ci.com/remi-dupre/dofus-generate-stuff)

Generate Dofus stuffs based on input requirements using a randomized local
search.


Installation
------------

### From releases

Automatic releases are published *for linux 64 bits*
[on GitHub](https://github.com/remi-dupre/dofus-generate-stuff/releases), you
can download an archive containing both the binary and required item databases.

```
wget https://github.com/remi-dupre/dofus-generate-stuff/releases/download/${VERSION}/dofus-generate-stuff.tar.gz
tar -xf dofus-generate-stuff.tar.gz
cd dofus-generate-stuff
./dofus-generate-stuff examples/earth_iop.json
```

### From sources

The only requirement is the Rust compilation toolchain, you can find
informations on how to install it from the
[official website](https://www.rust-lang.org/tools/install).

First, clone the repository.

```bash
git clone https://github.com/remi-dupre/dofus-generate-stuff.git
```

You will need to download a database of the equipment by using the command
bellow. Alternatively you may want to avoid waiting a few hours until this is
done and extract these files from an existing release (cf. previous section)
and put them in a directory called `data`.

```bash
./download_data.sh  # download informations about equipments
```

Once you downloaded data files, you can compile and run with a single cargo
command:

```bash
cargo run --release -- examples/earth_iop.json
```


Usage
-----

The values defined in the input don't need to be accurate, but only to give an
order of magnitude of the value expected in the output. If the requirements are
too strong, the result may be less ambitious, if they are not strong enough,
the result may be better than specified.

For example, someone who wants to build an annoying tanky build may target the
following kind of statistics:

```json
{
    "level": 200,
    "target", [
      ["Resiliance", 10000],
      [{"Carac": "Lock"}, 150],
      [{"Carac": "AP"}, 11],
      [{"Carac": "MP"}, 6],
      [{"PowStats": "Air"}, 800],
      [{"Carac": "MP Resistance"}, 70],
      [{"Carac": "AP Resistance"}, 70],
      [{"Carac": "AP Reduction"}, 100]
    ]
}
```

Which will output this build:

  ![](https://imgur.com/AFCaeqE.png)


How it works
------------

TODO
