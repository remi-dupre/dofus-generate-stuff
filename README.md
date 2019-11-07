Dofus Generate Stuff
====================

Generate Dofus stuffs based on input requirements using a randomized local
search.


Usage
-----

You can run the project quickly by using Cargo. You can change the default
input located in `input.json`.

```bash
./download_data.sh  # TODO
cargo run --release
```

### Input

The values defined in the input don't need to be accurate, but only to give an
order of magnitude of the value expected in the output. If the requirements are
too strong, the result may be less ambitious, if they are not strong enough,
the result may be better than specified.

For example, someone who wants to build an anoying tanky build may target the
following kind of statistics:

```json
[
  ["Resiliance", 10000],
  [{"Carac": "Lock"}, 150],
  [{"Carac": "AP"}, 11],
  [{"Carac": "MP"}, 6],
  [{"PowStats": "Air"}, 800],
  [{"Carac": "MP Resistance"}, 70],
  [{"Carac": "AP Resistance"}, 70],
  [{"Carac": "AP Reduction"}, 100]
]
```

Wich will output this build:

  ![](https://imgur.com/AFCaeqE.png)
