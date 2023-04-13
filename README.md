# Windows DSCP Fix Python Package

A Python package to fix `setsockopt(..,IP_TOS,..)` behavior on Windows.

### Why

- In the [IPPROTO_IP socket options](https://learn.microsoft.com/en-us/windows/win32/winsock/ipproto-ip-socket-options) for `setsockopt`, `IP_TOS` is marked as **"DO NOT USE"**, and [Windows QoS2 subsystem](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/qos/introduction-to-qos2--qwave-) takes over the admission.

- ToS/DSCP value is still an important field for QoS guarantee in IEEE 802.11 network, according to [RFC 8325](https://www.rfc-editor.org/rfc/rfc8325). However, there is no existing convenient method to tune the value.

- On Windows, only DSCP value is allowed to set except ECN bits and the Administrator permission is required ([link](https://learn.microsoft.com/en-us/windows/win32/api/qos2/nf-qos2-qossetflow)). So, this functionality has to be maintained as a third-party module, which will not be accepted by mainline.

### Usage

```python
import socket
import windows_dscp_fix
sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
sock.setsockopt(socket.IPPROTO_IP, socket.IP_TOS, 128) #NOTE: Administrator permission required
```