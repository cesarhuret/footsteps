"use client";

import React, { useState, useEffect } from "react";
import SelfQRcodeWrapper, { SelfApp, SelfAppBuilder } from "@selfxyz/qrcode";
import { v4 as uuidv4 } from "uuid";
import { logo } from "../content/playgroundAppLogo";
import { countryCodes } from "@selfxyz/core";
import { Inter } from 'next/font/google';
import Link from 'next/link';
 
const inter = Inter({
  subsets: ['latin'],
  variable: '--font-inter',
})

function Playground() {
  const [userId, setUserId] = useState<string | null>(null);
  const [savingOptions, setSavingOptions] = useState(false);

  useEffect(() => {
    setUserId(uuidv4());
  }, []);

  const [disclosures, setDisclosures] = useState({
    // DG1 disclosures
    issuing_state: false,
    name: false,
    nationality: true,
    date_of_birth: false,
    passport_number: false,
    gender: false,
    expiry_date: false,
    // Custom checks
    minimumAge: 18,
    excludedCountries: ["IRN", "IRQ", "PRK", "RUS", "SYR", "VEN"],
    ofac: true,
  });

  const [selectedCountries, setSelectedCountries] = useState<string[]>([
    countryCodes.IRN,
    countryCodes.IRQ,
    countryCodes.PRK,
    countryCodes.RUS,
    countryCodes.SYR,
    countryCodes.VEN,
  ]);

  const [countrySelectionError, setCountrySelectionError] = useState<
    string | null
  >(null);
  const [searchQuery, setSearchQuery] = useState("");

  const [endpoint, setEndpoint] = useState("");
  

  useEffect(() => {
    setEndpoint(localStorage.getItem("endpoint") || "");
  }, []);

  if (!userId) return null;

  const selfApp = new SelfAppBuilder({
    appName: "Footsteps",
    scope: "self-playground",
    endpoint: endpoint + "/api/verify",
    logoBase64: logo,
    userId,
    disclosures: {
      ...disclosures,
      minimumAge: 18,
    },
    devMode: false,
  } as Partial<SelfApp>).build();

  console.log("selfApp in:", selfApp);

  return (
    <div
      className="App flex flex-col min-h-screen bg-black text-white"
      suppressHydrationWarning
    >
      <div className="flex-1 flex flex-col items-center justify-center px-4 py-8">
        <div className="w-full max-w-3xl flex flex-col md:flex-row gap-2">
          <div className="w-full md:w-1/2 flex flex-col items-center justify-center">
          {
            endpoint ? 
            <SelfQRcodeWrapper
              selfApp={selfApp}
              onSuccess={() => {
                console.log("Verification successful");
                window.location.href = "/";
              }}
              darkMode={true}
            />

            : 
            <p className={`text-sm ${inter.variable} font-sans mb-4`}>No endpoint found. Please try again.</p>
          }
          </div>

          <div className="w-full md:w-1/2 p-8">
            <h2 className={`text-3xl ${inter.variable} font-sans mb-4`}>Player Verification</h2>
            <p className={`text-sm ${inter.variable} font-sans mb-4`}>Please verify using Self Protocol to continue.</p>

            <div className="space-y-6">
              <div className="h-full flex-col border border-gray-600 rounded-lg p-4 space-y-2">
                    <label className="flex items-center space-x-2">
                      <input
                        type="checkbox"
                        checked={true}
                        readOnly
                        className="h-4 w-4"
                      />
                      <span className={`${inter.variable} font-sans`}>
                        You are at least {disclosures.minimumAge} years old
                      </span>
                    </label>

                    <label className={`flex items-center space-x-2 ${inter.variable} font-sans`}>
                      <input
                        type="checkbox"
                        checked={true}
                        readOnly
                        className="h-4 w-4"
                      />
                      <span>
                        You are not from:
                      </span>
                    </label>
                    <div className="flex flex-col gap-2 ml-6 mt-2">
                    <ul>
                      {disclosures.excludedCountries.map((country: any) => (
                        <li key={country} className="text-sm text-gray-300">
                          <span>{(countryCodes as any)[country]}</span>
                        </li>
                      ))}
                    </ul>
                  </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

export default Playground;
