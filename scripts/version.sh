#!/usr/bin/env bash

version=""
yearweek=""
year=$(date +%Y)
weeknumber=$(date +%V) # ISO Standard week number

for ARGUMENT in "$@"; do
    KEY=$(echo $ARGUMENT | cut -f1 -d=)
    VALUE=$(echo $ARGUMENT | cut -f2 -d=)

    case "$KEY" in
    head) head=${VALUE} ;;
    build) build=${VALUE} ;;
    suffix) suffix=${VALUE} ;;
    override_version) override_version=${VALUE} ;;
    date) forced_date=${VALUE} ;;
    *) ;;
    esac
done

# region making yearweek
if [ ! -z ${forced_date} ]; then
    echo "Date is set to ${forced_date}"
    year=$(date -u -d ${forced_date} +%Y)
    weeknumber=$(date -d ${forced_date} +%V)
fi

# this prevents from having 1801 at the last week of the year 2019. It should be 1901.
if [[ ${weeknumber} -eq 1 ]] && [[ $(date -u -d ${forced_date} +%-d) -gt 20 ]]; then
    year=$(expr ${year} + 1)
fi

# this prevents from having 1053 at the last week of the year 2010. It should be 0953.
if [[ ${weeknumber} -ge 52 ]] && [[ $(date -u -d ${forced_date} +%-d) -le 7 ]]; then
    year=$(expr ${year} - 1)
fi

yearweek="${year:2:2}${weeknumber}"
# endregion making yearweek

last_version=$(git tag | tac | head -n 1)
last_yearweek=$(echo $last_version | cut -d '.' -f 2)

if [[ $last_yearweek -eq $yearweek ]]; then
    last_build=$(echo $last_version | cut -d '.' -f 3)
    build=$(expr $last_build + 1)
else
    build="0"
fi

if [ -z ${override_version} ]; then
    # By default, version is set to 0.
    if [ -z ${head} ]; then
        head="0"
        >&2 echo "- Warning: no head value. set to 0 by default."
    fi

    version="$head.$yearweek.$build"
else
    >&2 echo "- Warning: head, build, suffix values will be ignored"
    version=${override_version}
fi

# append suffix
if [ ! -z ${suffix} ]; then
    version="$version+$suffix"
fi

printf "$version"
