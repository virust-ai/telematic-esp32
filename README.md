# 🛰️ Telematic Platform for Robotics, EV, IoT.

## Overview

This open-source Telematic Platform for Robotics, EV.. is designed for collecting, processing, and transmitting CAN bus data over multiple connectivity options. It enables real-time monitoring, remote control, and OTA updates for ECUs, making it ideal for robotics, electric vehicles, and IoT applications.

This project is open for contributions! If you’re passionate about embedded systems, IoT, telematics, or robotics, we welcome you to collaborate, improve, and extend the platform.

# 🌟 Features
- CAN Bus Data Collection & Forwarding – Collects and transmits CAN messages to a cloud server.
- Remote Configuration – Configure data rates, schedules, and commands remotely from the server.
- OTA (Over-the-Air) Updates – Update firmware for ECUs via CAN.
- GPS/GNSS Location Tracking – Real-time geolocation support.
- Remote Commands from Server – Control ECUs from the cloud.
- Multi-Network Support – Works over Wi-Fi, Bluetooth, and LTE.
- IMU Sensor Integration – Tracks vibration and environmental factors (supports smell sensor integration).
- Fall & Crash Detection – Detects accidents and system failures.
- Remote Diagnostics – Enables fault analysis and debugging remotely.

# 📸 Hardware Overview
<img width="259" alt="image" src="https://github.com/user-attachments/assets/8cb6f342-93dc-4081-9f0b-baa21884126f" />

# Installation guide for Rust environment (Linux)
## Git
Fine-grained token is required to git clone in linux build environment
Instruction to get Fine-grained token can be obtained from 
https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/managing-your-personal-access-tokens
https://www.pragmaticlinux.com/2023/05/create-and-store-your-github-personal-access-token/
https://unixtutorial.org/how-to-generate-ed25519-ssh-key/

# Required IDE and build environment
## rust

To install the build environment, first run the following command 
```sudo apt-get install build-essential pkg-config libssl* libudev* ```
```curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh ```

then the installation guide will guide you through, just follow thru in default
and run command ```rustup update```


## espup
run command ```cargo install espup```
followed by ``` espup install```
and run this command once for include in the built environment ``` cat $HOME/export-esp.sh >> ~/.bashrc```

## esp-generate
this is to auto generate template for rust build. to install, run ```cargo install cargo-generate```
to create a project, run this command ```esp-generate --chip=esp32c6 your-project```

## esp-idf setup guide
follow this guide https://docs.espressif.com/projects/esp-idf/en/latest/esp32/get-started/linux-macos-setup.html#for-linux-users
run command ```sudo apt-get install git wget flex bison gperf python3 python3-pip python3-venv cmake ninja-build ccache libffi-dev libssl-dev dfu-util libusb-1.0-0 ```

```mkdir -p ~/esp```
```cd ~/esp```
```git clone --recursive https://github.com/espressif/esp-idf.git```

```cd ~/esp/esp-idf```
```./install.sh esp32```

```cd ~/esp/esp-idf```
```./install.sh esp32,esp32c6```


```alias get_idf='. $HOME/esp/esp-idf/export.sh'```

then ``` source ~/.profile ```
or edit the ~/.bashrc by sudo ```nano ~/.bashrc```
scroll to the bottom and add in ```. $HOME/esp/idf/export.sh```
and then ```source ~/.bashrc```
and try run idf.py 

## espflash
To flash build file into ESP32C6
run command ```cargo install espflash```
next followed by instruction in https://github.com/esp-rs/espflash/blob/main/espflash/README.md
run command ``` sudo usermod -a -G dialout $USER``` and ```su $USER```

## optional Wokwi Simulator
 follow guide to this link https://github.com/playfulFence/esp-hello-display/tree/feature/vscode-wokwi
