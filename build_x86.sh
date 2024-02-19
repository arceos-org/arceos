FILE=testsuits-x86_64-linux-musl

if [ ! -e testcases/$FILE ]; then
wget https://github.com/oscomp/testsuits-for-oskernel/releases/download/final-x86_64/$FILE.tgz
tar zxvf $FILE.tgz
mv $FILE testcases/$FILE -f
rm $FILE.tgz
fi
./build_img.sh $FILE