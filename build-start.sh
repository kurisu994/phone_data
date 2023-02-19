#!/bin/bash
git pull --rebase

#检查存在的镜像并打包镜像
docker images | grep phone-data &> /dev/null
if [ $? -eq 0 ]
then
    docker tag phone-data:lastest pre/phone-data:pre
fi

docker build -t phone-data:latest -f ./Dockerfile --no-cache .


#停止运行的容器
docker ps | grep phone-data &>/dev/null
if [ $? -eq 0 ]; then
  docker stop phone-data
fi

docker ps -a | grep phone-data &>/dev/null
if [ $? -eq 0 ]; then
  docker rm phone-data
fi

#启动容器
docker run -u root -d --name phone-data -p 9001:8080 phone-data:latest

#移除上一步的镜像
docker images | grep pre/phone-data &> /dev/null
if [ $? -eq 0 ]
then
    docker rmi pre/phone-data:pre
fi
