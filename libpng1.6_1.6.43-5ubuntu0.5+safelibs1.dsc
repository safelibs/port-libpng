Format: 3.0 (quilt)
Source: libpng1.6
Binary: libpng16-16t64, libpng-dev, libpng-tools, libpng16-16-udeb
Architecture: any
Version: 1.6.43-5ubuntu0.5+safelibs1
Maintainer: Ubuntu Developers <ubuntu-devel-discuss@lists.ubuntu.com>
Uploaders: Nobuhiro Iwamatsu <iwamatsu@debian.org>, Gianfranco Costamagna <locutusofborg@debian.org>, Tobias Frost <tobi@debian.org>
Homepage: http://libpng.org/pub/png/libpng.html
Standards-Version: 4.6.2
Vcs-Browser: https://salsa.debian.org/debian/libpng1.6
Vcs-Git: https://salsa.debian.org/debian/libpng1.6.git
Build-Depends: debhelper-compat (= 13), cargo, dpkg-dev (>= 1.22.5), gcc, mawk, rustc, zlib1g-dev
Package-List:
 libpng-dev deb libdevel optional arch=any
 libpng-tools deb libdevel optional arch=any
 libpng16-16-udeb udeb debian-installer optional arch=any profile=!noudeb
 libpng16-16t64 deb libs optional arch=any
Checksums-Sha1:
 7e8ad7c14b4aca0b860447e82cefd54331eeea37 190828 libpng1.6_1.6.43.orig.tar.xz
 3449557d9ae1a6a98a8b67b8ad42c8276dde5fb1 175364 libpng1.6_1.6.43-5ubuntu0.5+safelibs1.debian.tar.xz
Checksums-Sha256:
 245573d767b5374b12e0d261b69d38c48236b15581c5cf3de8b46caa494e4ba5 190828 libpng1.6_1.6.43.orig.tar.xz
 85e44ad3f87982154ab0487fb45386148eb8ed851c2b9a7e6e007f872374739d 175364 libpng1.6_1.6.43-5ubuntu0.5+safelibs1.debian.tar.xz
Files:
 17d02fb01d828e0cdc9e25389aae22d4 190828 libpng1.6_1.6.43.orig.tar.xz
 a0f2debb84fa625a228c41eb49cd60a3 175364 libpng1.6_1.6.43-5ubuntu0.5+safelibs1.debian.tar.xz
Original-Maintainer: Maintainers of libpng1.6 packages <libpng1.6@packages.debian.org>
