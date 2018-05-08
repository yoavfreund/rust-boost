export GIT_REPO="https://github.com/arapat/rust-boost.git"
export GIT_BRANCH="aws-scale"

sudo umount /mnt
yes | sudo mdadm --create --verbose /dev/md0 --level=0 --name=MY_RAID --raid-devices=2 /dev/xvdb /dev/xvdc
yes | sudo mkfs.ext4 -L MY_RAID /dev/md0
sudo mount LABEL=MY_RAID /mnt
sudo chown -R ubuntu /mnt

sudo apt-get install -y awscli

cd /mnt
aws s3 cp s3://ucsd-data/splice/testing.bin .
aws s3 cp s3://ucsd-data/splice/training.bin .

git config --global user.name "Julaiti Alafate"
git config --global user.email "jalafate@gmail.com"
git config --global push.default simple

echo "export EDITOR=vim" >> ~/.bashrc

git clone $GIT_REPO /mnt/rust-boost

sudo apt-get update
yes | sudo apt-get install cargo

