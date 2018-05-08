BASE_DIR="/home/ubuntu"
readarray -t nodes < $BASE_DIR/neighbors.txt

export IDENT_FILE="~/jalafate-dropbox.pem"

for i in "${!nodes[@]}";
do
    url=${nodes[$i]}
    echo "Checking $url..."

    if ssh -o StrictHostKeyChecking=no -i $IDENT_FILE $url test -f /mnt/training.bin \> /dev/null 2\>\&1
    then
        :
    else
        echo "!!! Training does not exists on $url."
    fi
done
