import {
    setPayerAndBlockhashTransaction,
    signAndSendTransaction,
    checkWallet,
    createCampaign,
    getAllCampaigns,
    donateToCampaign,
    withdraw,



  } from './index.js';
  
  async function main() {
    console.log("Let's say hello to a Solana account...");
  
    await setPayerAndBlockhashTransaction(instructions);
    await signAndSendTransaction(transaction);
    await checkWallet();
    await createCampaign(name, description, image_link);
    await getAllCampaigns();
    await donateToCampaign(campaignPubKey, amount);
    await withdraw(campaignPubKey, amount);
    
    console.log('Success');
  }
  
//   main().then(
//     () => process.exit(),
//     err => {
//       console.error(err);
//       process.exit(-1);
//     },
//   );