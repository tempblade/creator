[ -z $TOKEN ] && printf "Token is missing" && exit 1

if [ -z $IS_PROD ]; then
    printf "\nTEST DEPLOYMENT\n"
else
    printf "\nPRODUCTION DEPLOYMENT\n" && PROD="--prod"
fi

URL=$(vercel --yes --global-config ./.vercel --token $TOKEN $PROD) && printf "\nDEPLOYMENT SUCCESSFUL\n$URL"
