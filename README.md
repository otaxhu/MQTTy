# MQTTy

MQTTy it's a native MQTT GUI client written in Rust, that allows you to do what an MQTT client does, but in an easy and graphical way, with a focus on debugging and testing MQTT topics.

![Publish view](/data/screenshots/publish_view.png)

## Features:

- ### Publish MQTT messages

  Featuring an **embedded code editor**, you can use it to write any message you want to send to any MQTT topic you are connected to.

- ### Subscribe to MQTT topics

  You will receive system notifications when an incoming MQTT message arrives.

- ### Application runs on the background when you close it

  You can resume the application just by opening it again, it will keep notifying you of incoming MQTT messages when it's on the background.

- ### VCS-friendly and private-first

  You own your data, period, MQTTy is responsible for saving your data locally into a VCS-friendly format, so that you can share it with your development team.

## Downloads:

- ### Windows 10/11:
  Download the `MQTTy-X.Y.Z-win32-portable.zip` file from the [releases page](https://github.com/otaxhu/MQTTy/releases/latest), uncompress it and run the `bin/MQTTy.exe` executable.

- ### Flathub:
  <a href="https://flathub.org/apps/io.github.otaxhu.MQTTy">
    <img src="https://flathub.org/api/badge?svg&locale=en" alt="Get it from Flathub">
  </a>

## A little bit of history...

Imagine you are testing MQTT topics on any of the hundreds of Web MQTT clients, then you want to save all of the connection information for later use, oops! you can't, you need to create an account. You can't even save it locally, even if the browser allows to do it, at least that was my case.

On very early stages of development I wanted to create just that, another Web MQTT client, that would allow to save all of the data locally for the user to save as whatever he wanted, and then resume development by uploading that same file to the app.

But then I wanted a very important feature, **the application should run on the background**, I couldn't do it with a Web app.

That's why I started this project, I wanted the users to own his data and some nice features that weren't available on Web based apps.

## License

MQTTy is published under the terms of the GNU General Public License v3.0 or later versions.

    Copyright (c) 2025 Oscar Pernia

    MQTTy is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with this program.  If not, see <https://www.gnu.org/licenses/>.
