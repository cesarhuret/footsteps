import { NextApiRequest, NextApiResponse } from 'next';
import { 
    getUserIdentifier, 
    SelfBackendVerifier, 
    countryCodes 
} from '@selfxyz/core';

export default async function handler(req: NextApiRequest, res: NextApiResponse) {
    if (req.method === 'POST') {
        try {
            const { proof, publicSignals } = req.body;

            if (!proof || !publicSignals) {
                return res.status(400).json({ message: 'Proof and publicSignals are required' });
            }

            const userId = await getUserIdentifier(publicSignals);
            console.log("Extracted userId from verification result:", userId);

            // Default options
            const minimumAge = 18;
            const excludedCountryList: string[] = ["Iran (Islamic Republic of)", "Iraq", "Korea (Democratic People's Republic of)", "Russian Federation", "Syrian Arab Republic", "Venezuela (Bolivarian Republic of)"];
            const enableOfac = true;
            const enabledDisclosures = {
                issuing_state: false,
                name: false,
                nationality: true,
                date_of_birth: false,
                passport_number: false,
                gender: false,
                expiry_date: false
            };
            
            const configuredVerifier = new SelfBackendVerifier(
                'https://forno.celo.org',
                "self-playground",
                "uuid",
                true
            );
            
            if (minimumAge !== undefined) {
                configuredVerifier.setMinimumAge(minimumAge);
            }
            
            if (excludedCountryList.length > 0) {
                configuredVerifier.excludeCountries(
                    ...excludedCountryList as (keyof typeof countryCodes)[]
                );
            }
            
            if (enableOfac) {
                configuredVerifier.enableNameAndDobOfacCheck();
            }

            console.log(proof);
            console.log(publicSignals);

            const result = await configuredVerifier.verify(proof, publicSignals);
            console.log("Verification result:", result);

            if (result.isValid) {
                const filteredSubject = { ...result.credentialSubject };
                
                if (!enabledDisclosures.issuing_state && filteredSubject) {
                    filteredSubject.issuing_state = "Not disclosed";
                }
                if (!enabledDisclosures.name && filteredSubject) {
                    filteredSubject.name = "Not disclosed";
                }
                if (!enabledDisclosures.nationality && filteredSubject) {
                    filteredSubject.nationality = "Not disclosed";
                }
                if (!enabledDisclosures.date_of_birth && filteredSubject) {
                    filteredSubject.date_of_birth = "Not disclosed";
                }
                if (!enabledDisclosures.passport_number && filteredSubject) {
                    filteredSubject.passport_number = "Not disclosed";
                }
                if (!enabledDisclosures.gender && filteredSubject) {
                    filteredSubject.gender = "Not disclosed";
                }
                if (!enabledDisclosures.expiry_date && filteredSubject) {
                    filteredSubject.expiry_date = "Not disclosed";
                }
                
                res.status(200).json({
                    status: 'success',
                    result: result.isValid,
                    credentialSubject: filteredSubject,
                    verificationOptions: {
                        minimumAge,
                        ofac: enableOfac,
                        excludedCountries: excludedCountryList.map(countryName => {
                            const entry = Object.entries(countryCodes).find(([_, name]) => name === countryName);
                            return entry ? entry[0] : countryName;
                        })
                    }
                });
            } else {
                res.status(400).json({
                    status: 'error', 
                    result: result.isValid,
                    message: 'Verification failed',
                    details: result.isValidDetails
                });
            }
        } catch (error) {
            console.error('Error verifying proof:', error);
            return res.status(500).json({ 
                message: 'Error verifying proof',
                error: error instanceof Error ? error.message : 'Unknown error'
            });
        }
    } else {
        res.status(405).json({ message: 'Method not allowed' });
    }
}