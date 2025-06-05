# Jellyfin Discovery Util

I noticed that my Andrioid TV and Roku TV Jellyfin clients had the option to
discover a server over the LAN. Now this would be great for me because when I
bring a new client online, I would like my Jellyfin server to just be there.
But since this mechanism doesn't use mDNS or anything I could easily proxy
with my UniFi setup, I am making this to sit on a client on my LAN and reply
to auto discovery requests with my local server info.

This is **NOT** a relay (*yet?*) and requires manual configuration.

## How does Jellyfin Auto Discovery Work?

This was surprisingly difficult to decipher. I couldn't find much information
other than "Some Jellyfin clients can automatically discover servers on their
LAN." So I had to go digging in the source code and here is what I found:

1. Client Sends Broadcast

The client creates a UDP Limited Broadcast Datagram on port 7359 which just
contains the ASCII encoded text: "Who is JellyfinServer?"

```text
IP bf8d4ae90eca.51200 > 255.255.255.255.7359: UDP, length 22
        0x0000:  4500 0032 0a05 4000 4011 8497 ac12 000d  E..2..@.@.......
        0x0010:  ffff ffff c800 1cbf 001e ac4e 5768 6f20  ...........NWho.
        0x0020:  6973 204a 656c 6c79 6669 6e53 6572 7665  is.JellyfinServe
        0x0030:  723f                                     r?
```
2. Servers Respond to Request

The servers create a unicast UDP Datagram in reply to the client. This
datagram contains a JSON payload.

```text
IP jellyfin.docker_net.7359 > bf8d4ae90eca.51200: UDP, length 130
        0x0000:  4500 009e 68c9 4000 4011 7951 ac12 0003  E...h.@.@.yQ....
        0x0010:  ac12 000d 1cbf c800 008a 58d0 7b22 4164  ..........X.{"Ad
        0x0020:  6472 6573 7322 3a22 6874 7470 3a2f 2f31  dress":"http://1
        0x0030:  3732 2e31 382e 302e 333a 3830 3936 222c  72.18.0.3:8096",
        0x0040:  2249 6422 3a22 6462 3032 6164 6636 3435  "Id":"db02adf645
        0x0050:  3430 3430 6365 3865 6632 3535 3336 3439  4040ce8ef2553649
        0x0060:  6362 3230 6530 222c 224e 616d 6522 3a22  cb20e0","Name":"
        0x0070:  4465 6669 6e69 7465 6c79 204c 696e 7578  Definitely.Linux
        0x0080:  2049 534f 7322 2c22 456e 6470 6f69 6e74  .ISOs","Endpoint
        0x0090:  4164 6472 6573 7322 3a6e 756c 6c7d       Address":null}
```

Extracted the JSON looks something like this:

```json
{
  "Address": "http://172.18.0.3:8096",
  "Id": "db02adf6454040ce8ef2553649cb20e0",
  "Name": "Definitely Linux ISOs",
  "EndpointAddress": null
}
```

`Address` - This appears to just be the desired URL to provide the client.

`Id` - In the source code the variable is called `SystemId` and appears to be
uniquely generated when the server is first created. I don't think this has
any bearing on connectivity and seems to only be used to avoid duplicate
results in the server list. This server id is printed in the startup logs but
is not accessible anywhere in the WebUI.

`Name` - This is the server name set in the WebUI.

`EndpointAddress` - Appears to be deprecated (?). All the offical clients just
seem to drop it and the server makes no effort to set it in any conditions. It
may be a part of a previous version of auto discovery as the server referes to
this process as a V2Message.

## Getting Started

### Download

The latest version of the binary can be simply domloaded with:
```sh
# ARM (aarch64)
wget https://github.com/lukeh990/jellyfin-discovery-util/releases/latest/download/aarch64-jellyfin-discovery-util

# x86_64
wget https://github.com/lukeh990/jellyfin-discovery-util/releases/latest/download/x86_64-jellyfin-discovery-util
```

Remember that if you want to run it you need to do:
```sh
chmod +x {ARCH}-jellyfin-discovery-util
```

### Install

You can just manually run the binary however you want, but if you want to set
it up for automatic running, here is a guide.

1. Move binary to `/usr/bin/local`

```sh
sudo mv {ARCH}-jellyfin-discovery-util /usr/local/bin/jellyfin-discovery-util
sudo chown root:root /usr/local/bin/jellyfin-discovery-util
```

2. Create a system user for the service

```sh
sudo useradd -M -r jd-util
```
This ceates a new system user (`-r`) with no home directory (`-M`) called 
`jd-util`. This allows us to protect the root account by running as a less
privileged user.

3. Create a directory for our config

```sh
sudo mkdir /etc/jd-util
```

If you already made a config put it in here. If you don't a sample config will
be made for you at `/etc/jd-util/discover.toml`

```sh
sudo chown -R jd-util:jd-util /etc/jd-util
```

Make sure that our system user can access the config and the directory.

4. Create a systemd unit

I put my systemd unit in this file: `/etc/systemd/system/jd-util.service`

```systemd
[Unit]
Description=Jellyfin Discovery Utility
After=network.target

[Service]
Type=simple
User=jd-util
Group=jd-util
WorkingDirectory=/etc/jd-util
ExecStart=/usr/local/bin/jellyfin-discovery-util
Restart=on-failure
RestartSec=5s

[Install]
WantedBy=multi-user.target
```

After saving this file, run these commands to setup allow the service to start
on boot.

```sh
sudo systemctl daemon-reload
sydo systemctl enable --now jd-util
```

The binary will automatically integrate with journald and will properly send
logs which can be view with either

```sh
sudo systemctl status jd-util

# OR

sudo journalctl -u jd-util.service
```

## License

This project is MIT Licensed. 
