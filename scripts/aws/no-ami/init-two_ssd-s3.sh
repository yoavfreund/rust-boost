AWS_KEY_FILE=/home/ubuntu/awskey.sh

sudo umount /mnt
yes | sudo mdadm --create --verbose /dev/md0 --level=0 --name=MY_RAID --raid-devices=2 /dev/xvdb /dev/xvdc
sudo mkfs.ext4 -L MY_RAID /dev/md0
sudo mount LABEL=MY_RAID /mnt
sudo chown -R ubuntu /mnt

yes | sudo apt-get install awscli

source $AWS_KEY_FILE
cd /mnt
aws s3 cp s3://ucsd-data/splice/testing.bin .
aws s3 cp s3://ucsd-data/splice/training.bin .

# Clone repository
ssh -o StrictHostKeyChecking=no -i $IDENT_FILE ubuntu@$url git clone $GIT_REPO /mnt/rust-boost

# Install cargo
ssh -o StrictHostKeyChecking=no -i $IDENT_FILE ubuntu@$url sudo apt-get update
ssh -o StrictHostKeyChecking=no -i $IDENT_FILE ubuntu@$url sudo apt-get install -y cargo

