#!/bin/sh

# SPDX-License-Identifier: MPL-2.0

set -e

# 测试IPv4 TCP连接（已完成）
./tcp_server &
sleep 0.2
./tcp_client

# 测试IPv6 TCP连接（已完成）
./tcp6_server &
sleep 0.2
./tcp6_client

# 测试IPv4 UDP连接（已完成）
./udp_server &
sleep 0.2
./udp_client

# 跳过IPv6 UDP测试（未完成）
# ./udp6_server &
# sleep 0.2
# ./udp6_client

# 测试Unix域套接字
./unix_server &
sleep 0.2
./unix_client

# 测试其他网络功能
./socketpair
./sockoption
./sockoption_unix
./listen_backlog
./send_buf_full
./tcp_err
./tcp_poll
./tcp_reuseaddr
./udp_broadcast
./udp_err
./unix_stream_err
./unix_seqpacket_err
./unix_datagram_err
./sendmmsg

# 测试网络链路
./netlink_route
./rtnl_err
./uevent_err
