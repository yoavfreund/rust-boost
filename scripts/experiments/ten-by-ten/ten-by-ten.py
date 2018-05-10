TYPE = 0

import subprocess

with open("./neighbors.txt") as f:
    all_neighbors = f.readlines()
all_neighbors = [s.strip() for s in all_neighbors if s.strip()]
assert(len(all_neighbors) == 100)

for i in range(10):
    with open("./neighbors-{}.txt".format(i), "w") as f:
        f.write('\n'.join(all_neighbors[i*10:i*10 + 10]))

    head_machine = all_neighbors[i * 10]
    command = (
        "scp-dropbox ~/Dropbox/documents/vault/aws/jalafate-dropbox.pem"
        "~/Downloads/credentials ubuntu@{}:~"
    ).fomrat(head_machine)
    subprocess.run(command.split())
    command = "scp-dropbox ./neighbors-{}.txt ubuntu@{}:~".format(i, head_machine)
    subprocess.run(command.split())

    command = (
        "ssh-dropbox ubuntu@{} /mnt/rust-boost/scripts/gen-config.sh".format(head_machine)
    )
    subprocess.run(command.split())

    if TYPE == 0:
        command = (
            "ssh-dropbox ubuntu@{} /mnt/rust-boost/scripts/experiments/parallel-on-network.sh".format(
                head_machine)
        )
        subprocess.run(command.split())
    elif TYPE == 1:
        command = (
            "ssh-dropbox ubuntu@{} /mnt/rust-boost/scripts/experiments/parallel-on-network.sh".format(
                head_machine)
        )
        subprocess.run(command.split())
    elif TYPE == 2:
        command = (
            "ssh-dropbox ubuntu@{} /mnt/rust-boost/scripts/experiments/parallel-on-network.sh".format(
                head_machine)
        )
        subprocess.run(command.split())