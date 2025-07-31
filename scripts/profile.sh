#!/usr/bin/env bash

set -e

host=$1

ssh -q $host << EOF
export mcp_pid=\$(ps aux | grep '/opt/keipes-ai-mcp/main' | grep -v 'grep' | awk '{print \$2}')
echo "got pid \$mcp_pid"
sudo /home/ec2-user/.cargo/bin/flamegraph -o /home/ec2-user/my_flamegraph.svg --pid \$mcp_pid
sudo chown ec2-user /home/ec2-user/my_flamegraph.svg
EOF

local_name=my_flamegraph_$(date +%s).svg
scp $host:/home/ec2-user/my_flamegraph.svg $local_name
# open $local_name


sudo /home/ec2-user/.cargo/bin/flamegraph -o /home/ec2-user/my_flamegraph.svg --pid $(ps aux | grep '/opt/keipes-ai-mcp/main' | grep -v 'grep' | awk '{print $2}')

scp mcp:/home/ec2-user/my_flamegraph.svg flamegraph_$(date +%s).svg