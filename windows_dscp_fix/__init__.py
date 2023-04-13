import functools
import socket
from windows_dscp_fix import windows_dscp_fix

@functools.wraps(socket.socket.setsockopt)
def __setsockopt(_self:socket.socket, level:int, optname:int, *args, **kwargs):
    if optname==socket.IP_TOS:
        fd, value = _self.fileno(), args[0]
        windows_dscp_fix.set_socket_tos(fd, value)
    else:
        _self.setsockopt(level, optname, *args, **kwargs)
    pass

socket.socket.setsockopt = __setsockopt
