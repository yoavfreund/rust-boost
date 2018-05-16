export GIT_REPO="https://github.com/arapat/rust-boost.git"
export GIT_BRANCH="aws-scale"

export DISK="/dev/xvdb"
sudo umount /mnt
yes | sudo mkfs.ext4 $DISK
sudo mount $DISK /mnt
sudo chown -R ubuntu /mnt

sudo apt-get update
sudo apt-get install -y awscli

cd /mnt

git config --global user.name "Julaiti Alafate"
git config --global user.email "jalafate@gmail.com"
git config --global push.default simple

echo "export EDITOR=vim" >> ~/.bashrc

git clone $GIT_REPO /mnt/rust-boost

yes | sudo apt-get install cargo

wait 5

mkdir ~/.aws
cp ~/credentials ~/.aws/
aws s3 cp s3://ucsd-data/splice/testing.bin .
aws s3 cp s3://ucsd-data/splice/training.bin .
