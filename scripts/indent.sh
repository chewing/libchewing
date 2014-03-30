#!/bin/bash
ROOTDIR=$(dirname $0)/..

echo -n "Find indent ... "
if ! which indent; then
    echo "not found";
    exit 1;
fi

find \
    ${ROOTDIR}/src \
    ${ROOTDIR}/test \
    ${ROOTDIR}/include \
    -regex ".*\.[ch]$" \
    -exec indent \
        --blank-lines-after-declarations \
        --blank-lines-after-procedures \
        --braces-on-if-line \
        --braces-on-struct-decl-line \
        --break-before-boolean-operator \
        --case-indentation 0 \
        --comment-indentation 33 \
        --continuation-indentation 4 \
        --continue-at-parentheses \
        --cuddle-else \
        --declaration-comment-column 33 \
        --declaration-indentation 1 \
        --dont-break-procedure-type \
        --dont-format-comments \
        --dont-format-first-column-comments \
        --dont-space-special-semicolon \
        --dont-star-comments \
        --else-endif-column 33 \
        --honour-newlines \
        --indent-level 4 \
        --leave-optional-blank-lines \
        --line-comments-indentation 0 \
        --line-length 120 \
        --no-blank-lines-after-commas \
        --no-comment-delimiters-on-blank-lines \
        --no-space-after-function-call-names \
        --no-space-after-parentheses \
        --no-tabs \
        --parameter-indentation 0 \
        --preprocessor-indentation 4 \
        --space-after-cast \
        --space-after-for \
        --space-after-if \
        --space-after-while \
        -T AvailInfo \
        -T BufferType \
        -T Category \
        -T ChewingContext \
        -T ChewingData \
        -T ChewingOutput \
        -T ChewingStaticData \
        -T ChoiceInfo \
        -T HASH_ITEM \
        -T keymap \
        -T Phrase \
        -T PhraseData \
        -T PhraseData \
        -T PhraseIntervalType \
        -T PhraseIntervalType \
        -T PhrasingOutput \
        -T PinYinData \
        -T plat_mmap \
        -T PreeditBuf \
        -T RecordNode \
        -T RecordNode \
        -T SqlStmtConfig_ \
        -T SqlStmtUserphrase_ \
        -T SymbolEntry \
        -T TestData \
        -T _tNODE \
        -T TreeDataType \
        -T TreeDataType \
        -T TreeType \
        -T uint16_t \
        -T UserPhraseData \
        -T BopomofoData \
        {} \;
