#!/bin/bash
git clone --branch usability \
    https://github.com/remi-dupre/crawlit-dofus-encyclopedia-parser.git \
    crawlit

(cd crawlit \
    && npm install \
    && npm run crawlit \
    && node ./dist/app.js -g dofus -l english -c allequipments --all \
    && node ./dist/app.js -g dofus -l english -c allweapons --all \
    && node ./dist/app.js -g dofus -l english -c mount --all \
    && node ./dist/app.js -g dofus -l english -c pet --all \
    && node ./dist/app.js -g dofus -l english -c set --all )

mkdir -p data
cp crawlit/data/dofus/allequipments.json data/equipments.json
cp crawlit/data/dofus/allweapons.json    data/weapons.json
cp crawlit/data/dofus/mount.json         data/mounts.json
cp crawlit/data/dofus/pet.json           data/pets.json
cp crawlit/data/dofus/set.json           data/sets.json

rm -rf crawlit
