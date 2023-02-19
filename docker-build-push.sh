#!/bin/bash

version=$1

docker images | grep kurisu003/phone-data &> /dev/null

if [ $? -eq 0 ]
then
    docker tag kurisu003/phone-data:lastest kurisu003/phone-data:$version
    docker push kurisu003/phone-data:$version
    docker rmi kurisu003/phone-data:lastest
    docker rmi kurisu003/phone-data:$version
fi

docker build -t phone-data:latest -f ./Dockerfile --no-cache .

if [ $? -eq 0 ]
then
    docker tag phone-data:latest kurisu003/phone-data:latest &&  docker push kurisu003/phone-data:lastest
fi
