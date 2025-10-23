#!/bin/bash
sudo docker rmi -f $(sudo docker images -q)
# remove the current binary file
rm powchain
# copy the new binary file
cp ../target/debug/powchain ./

# build the new image, the new image will replace the old one
sudo docker build --no-cache -t optchain .

# tag the image of its version, with the name of docker account
sudo docker tag optchain hkustelric/optchain:debug

# login the docker account (automatically)
sudo docker login

# push the new image to the depository
sudo docker push hkustelric/optchain:debug