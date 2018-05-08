IDENT_FILE=/home/ubuntu/jalafate-dropbox.pem
AWS_KEY_FILE=/home/ubuntu/awskey.sh

if [ ! -f $IDENT_FILE ]; then
    echo "Identification file not found!"
    exit 1
fi
if [ ! -f $AWS_KEY_FILE ]; then
    echo "AWS credential file not found!"
    exit 1
fi

BASE_DIR="/mnt"
export INIT_SCRIPT=$BASE_DIR/rust-boost/scripts/aws/no-ami/init-two_ssd-s3.sh
export GIT_REPO="https://github.com/arapat/rust-boost.git"
export GIT_BRANCH="aws-scale"

readarray -t nodes < $BASE_DIR/neighbors.txt

if [ $1 = "init" ]; then
    echo "Initialize all computers. Are you sure? (y/N)"
    read yesno
    if [[ "$yesno" != "y" ]] ; then
        echo "Aborted."
        exit 1
    fi

    for i in "${!nodes[@]}";
    do
        url=${nodes[$i]}
        echo
        echo "===== Initializing $url ====="
        echo

        if ssh -o StrictHostKeyChecking=no -i $IDENT_FILE $url test -f /mnt/init-done.txt \> /dev/null 2\>\&1
        then
            echo "The node has been initialized. Skipped."
        else
            # Copy init script
            scp -o StrictHostKeyChecking=no -i $IDENT_FILE $INIT_SCRIPT ubuntu@$url:~/init.sh

            # Execute init script
            ssh -o StrictHostKeyChecking=no -i $IDENT_FILE ubuntu@$url "bash ~/init.sh > /dev/null 2>&1 < /dev/null &"

            ssh -o StrictHostKeyChecking=no -i $IDENT_FILE ubuntu@$url touch /mnt/init-done.txt
            echo "Initialization is started."
        fi
    done

    echo
    echo "Now waiting for training/testing files to be transmitted to all other computers..."
fi

