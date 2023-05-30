# Using the web extension

This section describes how to use the Penumbra Wallet web extension, a GUI client for Penumbra.

## Installing the extension

The Penumbra Wallet web extension only supports the Google Chrome browser.
You must run Chrome in order to follow the instructions below.

1. Visit the [Web Store page for the Penumbra Wallet](https://chrome.google.com/webstore/detail/penumbra-wallet/lkpmkhpnhknhmibgnmmhdhgdilepfghe),
   and click **Add to Chrome** to install it.
2. Navigate to the dApp website for the extension: [https://app.testnet.penumbra.zone/](https://app.testnet.penumbra.zone/) and click **Connect** in the top-right corner.
3. Click **Get started** to proceed with wallet configuration.

## Generating a wallet
You'll be offered to import a pre-existing wallet. If you don't already have one, choose **Create a new wallet**.
During the guided tutorial, you'll need to set a passphrase to protect your wallet. The passphrase
is *not* the same as the recovery phrase. The passphrase is used to restrict access to the web wallet
on your computer. The recovery phrase can be used to import your wallet on a fresh installation, or
ona  different machine. Make sure to store both the passphrase and the recovery phrase
securely, for example in a password manager.

Re-enter portions of the recovery phrase when prompted, to confirm that you've saved it properly.
Then you'll be taken to a screen that shows an initial synchronization process with the most
recent testnet:

<!--
Do we want to maintain screenshots inside the web extension docs?
The image files will become out of data quickly, requiring maitnenance, and bloat the repo.
-->

<picture>
  <source srcset="../web-extension-sync-progress.png" media="(prefers-color-scheme: dark)" />
  <img src="../web-extension-sync-progress.png" />
</picture>

## Creating transactions
Now that you've got the web wallet configured, let's use it to send a transaction.
Navigate to the dApp website: [https://app.testnet.penumbra.zone/](https://app.testnet.penumbra.zone/) and click **Connect**,
then authorize the extension to work with the site. After doing so, you'll see buttons for actions
such as **Receive**, **Send**, and **Exchange**.

As of Testnet 53, only the **Send** action is supported. Check back on subsequent versions to follow progress
as we implement more advanced functionality in the web wallet.

If you don't have any funds in your wallet yet, see the [pcli docs](../pcli/wallet.md) for how to use our
testnet token faucet.
