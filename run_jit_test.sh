#!/bin/bash

echo "Note: this script is only for the convenience of running tests in our lab. "

lowercase(){
    echo "$1" | sed "y/ABCDEFGHIJKLMNOPQRSTUVWXYZ/abcdefghijklmnopqrstuvwxyz/"
}

# detect OS
OS=`lowercase \`uname\``
KERNEL=`uname -r`
MACH=`uname -m`

if [ "{$OS}" == "windowsnt" ]; then
    OS=windows
elif [ "{$OS}" == "darwin" ]; then
    OS=mac
else
    OS=`uname`
    if [ "${OS}" = "SunOS" ] ; then
        OS=Solaris
        ARCH=`uname -p`
        OSSTR="${OS} ${REV}(${ARCH} `uname -v`)"
    elif [ "${OS}" = "AIX" ] ; then
        OSSTR="${OS} `oslevel` (`oslevel -r`)"
    elif [ "${OS}" = "Linux" ] ; then
        if [ -f /etc/redhat-release ] ; then
            DistroBasedOn='RedHat'
            DIST=`cat /etc/redhat-release |sed s/\ release.*//`
            PSUEDONAME=`cat /etc/redhat-release | sed s/.*\(// | sed s/\)//`
            REV=`cat /etc/redhat-release | sed s/.*release\ // | sed s/\ .*//`
        elif [ -f /etc/SuSE-release ] ; then
            DistroBasedOn='SuSe'
            PSUEDONAME=`cat /etc/SuSE-release | tr "\n" ' '| sed s/VERSION.*//`
            REV=`cat /etc/SuSE-release | tr "\n" ' ' | sed s/.*=\ //`
        elif [ -f /etc/mandrake-release ] ; then
            DistroBasedOn='Mandrake'
            PSUEDONAME=`cat /etc/mandrake-release | sed s/.*\(// | sed s/\)//`
            REV=`cat /etc/mandrake-release | sed s/.*release\ // | sed s/\ .*//`
        elif [ -f /etc/debian_version ] ; then
            DistroBasedOn='Debian'
            DIST=`cat /etc/lsb-release | grep '^DISTRIB_ID' | awk -F=  '{ print $2 }'`
            PSUEDONAME=`cat /etc/lsb-release | grep '^DISTRIB_CODENAME' | awk -F=  '{ print $2 }'`
            REV=`cat /etc/lsb-release | grep '^DISTRIB_RELEASE' | awk -F=  '{ print $2 }'`
        fi
        if [ -f /etc/UnitedLinux-release ] ; then
            DIST="${DIST}[`cat /etc/UnitedLinux-release | tr "\n" ' ' | sed s/VERSION.*//`]"
        fi
        OS=`lowercase $OS`
        DistroBasedOn=`lowercase $DistroBasedOn`
        readonly OS
        readonly DIST
        readonly DistroBasedOn
        readonly PSUEDONAME
        readonly REV
        readonly KERNEL
        readonly MACH
    fi
fi

echo "---------"
echo "OS: $OS"
echo "KERNEL: $KERNEL"
echo "ARCH: $MACH"
echo "---------"

echo "building zebu..."

rm -rf emit/*

if [ "$OS" == "linux" ]; then
	RUSTFLAGS=-Zincremental=target/incr-cache RUST_BACKTRACE=1 CC=clang-3.8 cargo build
elif [ "$OS" == "Darwin" ]; then
	RUSTFLAGS=-Zincremental=target/incr-cache RUST_BACKTRACE=1 cargo build
else
	echo "unknown OS. do not use this script to run"
	exit
fi

echo ""
echo "fetching rpython..."
git clone https://gitlab.anu.edu.au/mu/mu-client-pypy.git mu-client-pypy

echo ""
echo "compiling rpython..."

# save current path
project_path=$(pwd)

cd mu-client-pypy
git checkout test-mu-impl-jit
cd rpython/translator/mu/rpyc; make
cd $project_path/tests/test_jit

if [ "$OS" == "linux" ]; then
	if [ $# -eq 0 ]; then
		RUST_BACKTRACE=1 PYTHONPATH=$project_path/mu-client-pypy MU_RUST=$project_path CC=clang-3.8 python -m pytest . -v
	else
		RUST_BACKTRACE=1 PYTHONPATH=$project_path/mu-client-pypy MU_RUST=$project_path CC=clang-3.8 python -m pytest . -v -k $@
	fi

elif [ "$OS" == "Darwin" ]; then
	if [ $# -eq 0 ]; then
		RUST_BACKTRACE=1 PYTHONPATH=$project_path/mu-client-pypy MU_RUST=$project_path CC=clang python2 -m pytest . -v
	else
		RUST_BACKTRACE=1 PYTHONPATH=$project_path/mu-client-pypy MU_RUST=$project_path CC=clang python2 -m pytest . -v -k $@
	fi
else
	echo "unknown OS. do not use this script to run"
	exit
fi