#!/bin/bash
git clone \
    https://github.com/dofapi/crawlit-dofus-encyclopedia-parser.git \
    crawlit
cd crawlit
npm install
npm run dofus-all-en
# rm -rf crawlit
