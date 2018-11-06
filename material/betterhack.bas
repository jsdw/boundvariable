V           REM  +------------------------------------------------+
X           REM  | HACK.BAS      (c) 19100   fr33 v4r14bl3z       |
XV          REM  |                                                |
XX          REM  | Brute-forces passwords on UM vIX.0 systems.    |
XXV         REM  | Compile with Qvickbasic VII.0 or later:        |
XXX         REM  |    /bin/qbasic hack.bas                        |
XXXV        REM  | Then run:                                      |
XL          REM  |   ./hack.exe username                          |
XLV         REM  |                                                |
L           REM  | This program is for educational purposes only! |
LV          REM  +------------------------------------------------+
LX          REM
LXV         IF ARGS() > I THEN GOTO LXXXV
LXX         PRINT "usage: ./hack.exe username"
LXXV        PRINT CHR(X)
LXXX        END
LXXXV       REM
XC          REM  get username from command line
XCV         DIM username AS STRING
C           username = ARG(II)
CV          REM  common words used in passwords
CX          DIM pwdcount AS INTEGER
CXV         pwdcount = LIII
CXX         DIM words(pwdcount) AS STRING
CXXV        words(I) = "airplane"
CXXX        words(II) = "alphabet"
CXXXV       words(III) = "aviator"
CXL         words(IV) = "bidirectional"
CXLV        words(V) = "changeme"
CL          words(VI) = "creosote"
CLV         words(VII) = "cyclone"
CLX         words(VIII) = "december"
CLXV        words(IX) = "dolphin"
CLXX        words(X) = "elephant"
CLXXV       words(XI) = "ersatz"
CLXXX       words(XII) = "falderal"
CLXXXV      words(XIII) = "functional"
CXC         words(XIV) = "future"
CXCV        words(XV) = "guitar"
CC          words(XVI) = "gymnast"
CCV         words(XVII) = "hello"
CCX         words(XVIII) = "imbroglio"
CCXV        words(XIX) = "january"
CCXX        words(XX) = "joshua"
CCXXV       words(XXI) = "kernel"
CCXXX       words(XXII) = "kingfish"
CCXXXV      words(XXIII) = "(\b.bb)(\v.vv)"
CCXL        words(XXIV) = "millennium"
CCXLV       words(XXV) = "monday"
CCL         words(XXVI) = "nemesis"
CCLV        words(XXVII) = "oatmeal"
CCLX        words(XXVIII) = "october"
CCLXV       words(XXIX) = "paladin"
CCLXX       words(XXX) = "pass"
CCLXXV      words(XXXI) = "password"
CCLXXX      words(XXXII) = "penguin"
CCLXXXV     words(XXXIII) = "polynomial"
CCXC        words(XXXIV) = "popcorn"
CCXCV       words(XXXV) = "qwerty"
CCC         words(XXXVI) = "sailor"
CCCV        words(XXXVII) = "swordfish"
CCCX        words(XXXVIII) = "symmetry"
CCCXV       words(XXXIX) = "system"
CCCXX       words(XL) = "tattoo"
CCCXXV      words(XLI) = "thursday"
CCCXXX      words(XLII) = "tinman"
CCCXXXV     words(XLIII) = "topography"
CCCXL       words(XLIV) = "unicorn"
CCCXLV      words(XLV) = "vader"
CCCL        words(XLVI) = "vampire"
CCCLV       words(XLVII) = "viper"
CCCLX       words(XLVIII) = "warez"
CCCLXV      words(XLIX) = "xanadu"
CCCLXX      words(L) = "xyzzy"
CCCLXXV     words(LI) = "zephyr"
CCCLXXX     words(LII) = "zeppelin"
CCCLXXXV    words(LIII) = "zxcvbnm"
CCCXC       REM try each password
CCCXCV      PRINT "attempting hack with " + pwdcount + " passwords " + CHR(X)
CD          DIM i AS INTEGER
CDV         i = I
CDX         IF CHECKPASS(username, words(i)) THEN GOTO CDXXX
CDXV        i = i + I
CDXX        IF i > pwdcount THEN GOTO CDXLV
CDXXV       GOTO CDX
CDXXX       PRINT "found match!! for user " + username + CHR(X)
CDXXXV      PRINT "password: " + words(i) + CHR(X)
CDXL        END
CDXLV       PRINT "no simple matches for user " + username + CHR(X)
CDL         REM
CDLV        REM  the above code will probably crack passwords for many
CDLX        REM  users so I always try it first. when it fails, I try the
CDLXV       REM  more expensive method below.
CDLXX       REM
CDLXXV      REM  passwords often take the form
CDLXXX      REM    dictwordDD
CDLXXXV     REM  where DD is a two-digit decimal number. try these next:
CDXC        i = I
CDXCV       REM
CDXCX       REM Declare 2 numbers, increment each in inner + outer loop
CDXCXV      REM and check password corresponding to them. goto end if done
CDXCXX      REM to print the password.
CDXCXXV     REM
CDXCXXX     DIM decimals(X) AS STRING
CDXCXXXV    decimals(I) = "0"
CDXCXL      decimals(II) = "1"
CDXCXLV     decimals(III) = "2"
CDXCL       decimals(IV) = "3"
CDXCLV      decimals(V) = "4"
CDXCLX      decimals(VI) = "5"
CDXCLXV     decimals(VII) = "6"
CDXCLXX     decimals(VII) = "7"
CDXCLXXV    decimals(IX) = "8"
CDXCLXXX    decimals(X) = "9"
CDXCLXXXV   DIM j AS INTEGER
CDXCXC      DIM k AS INTEGER
CDXCXCV     j = I
CDXCC       k = I
CDXCCV      IF CHECKPASS(username, words(i) + decimals(j) + decimals(k)) THEN GOTO CDXCCLXL
CDXCCX      j = j + I
CDXCCXV     IF j < XI THEN GOTO CDXCCV
CDXCCXX     j = I
CDXCCXXV    k = k + I
CDXCCXXX    IF k < XI THEN GOTO CDXCCV
CDXCCXXXV   i = i + I
CDXCCXL     IF i > pwdcount THEN GOTO CDXCCLXV
CDXCCXLV    GOTO CDXCCV
CDXCCL      REM
CDXCCLV     REM We failed to find a password like this so give up:
CDXCCLX     REM
CDXCCLXV    PRINT "no complex matches for user " + username + CHR(X)
CDXCCLXX    END
CDXCCLXXV   REM
CDXCCLXXX   REM We succeeded at finding a complex PW so print it:
CDXCCLXXXV  REM
CDXCCLXL    PRINT "found complex match!! for user " + username + CHR(X)
CDXCCLXLV   PRINT "password: " + words(i) + decimals(j) + decimals(k)
CDXCCLL     END




