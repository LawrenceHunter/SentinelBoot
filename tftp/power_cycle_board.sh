#!/bin/bash

if [[ $1 == "on" ]];
then
    usbrelay HURTM_1=0 && usbrelay HURTM_2=0
elif [[ $1 == "off" ]];
then
    usbrelay HURTM_1=1 && usbrelay HURTM_2=1
fi
