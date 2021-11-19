FROM tendermint/tendermint:latest

# Elevate to root permissions to install python3 to run the patch script
USER root
RUN apk add python3
USER tmuser

# Initialize a new Tendermint validator
# XXX NOTE will wipe away any existing data, this is NOT SAFE
# for real releases
RUN tendermint unsafe-reset-all && tendermint init validator

# Now a genesis file has been created at /tendermint/config/genesis.json
# and we can insert the genesis app state data
COPY patch_genesis.py .
RUN python3 patch_genesis.py && rm patch_genesis.py

# Delete Python3 since it's no longer needed in the image
USER root
RUN apk del python3
USER tmuser
