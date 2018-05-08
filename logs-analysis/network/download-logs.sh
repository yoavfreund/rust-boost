DOWNLOAD_PATH="/hdd/nips/may-08/1-node"
IDENT="~/Dropbox/documents/vault/aws/jalafate-dropbox.pem"
LOG_PATH="/mnt/rust-boost/*.log"
MODEL_PATH="/mnt/rust-boost/model*.json"
PICKLE_PATH="/mnt/rust-boost/*.pkl"


readarray -t nodes < ./neighbors.txt

cd $DOWNLOAD_PATH

for i in "${!nodes[@]}";
do
    echo ${nodes[$i]}
    mkdir node-$i
    scp -o StrictHostKeyChecking=no -i $IDENT ubuntu@${nodes[i]}:$LOG_PATH node-$i/ &
    scp -o StrictHostKeyChecking=no -i $IDENT ubuntu@${nodes[i]}:$MODEL_PATH node-$i/ &
    scp -o StrictHostKeyChecking=no -i $IDENT ubuntu@${nodes[i]}:$PICKLE_PATH node-$i/ &
done

wait

