import React from "react";
import { DigaTab } from "../types/domain";
import { Cpu, HardDrive, Key, Terminal, BookOpen } from "lucide-react";

interface NavbarProps {
  readonly currentTab: DigaTab;
  readonly onSelectTab: (tab: DigaTab) => void;
}

export const Navbar: React.FC<NavbarProps> = ({ currentTab, onSelectTab }) => {
  const navItems = [
    {
      tab: DigaTab.NVRAM_SANITIZER,
      label: "NVRAM Sanitizer (U99 Fix)",
      icon: Cpu,
    },
    {
      tab: DigaTab.HDD_FORENSICS,
      label: "MEIHDFS HDD Forensics",
      icon: HardDrive,
    },
    {
      tab: DigaTab.FRM_CRYPTANALYSIS,
      label: "FRM Container & ECB",
      icon: Key,
    },
    {
      tab: DigaTab.DWIRE_PICO_BRIDGE,
      label: "DWire32A Pico In-Circuit",
      icon: Terminal,
    },
    {
      tab: DigaTab.DOCUMENTATION_SEARCH,
      label: "Manuals & Schematics",
      icon: BookOpen,
    },
  ];

  return (
    <header className="border-b border-gray-800 bg-gray-900/90 backdrop-blur sticky top-0 z-50">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex items-center justify-between h-16">
          <div className="flex items-center space-x-3">
            <div className="w-10 h-10 rounded-lg bg-blue-600 flex items-center justify-center font-bold text-lg text-white shadow-lg shadow-blue-500/20">
              P
            </div>
            <div>
              <h1 className="text-lg font-bold tracking-tight text-white flex items-center gap-2">
                PANASONIC DIGA <span className="text-xs font-semibold px-2 py-0.5 rounded bg-blue-900/60 text-blue-300 border border-blue-700">DMR-EH55 SUITE</span>
              </h1>
              <p className="text-xs text-gray-400">Engineering & Systems Hardware Forensics</p>
            </div>
          </div>
          <nav className="flex space-x-1">
            {navItems.map((item) => {
              const Icon = item.icon;
              const isActive = currentTab === item.tab;
              return (
                <button
                  key={item.tab}
                  onClick={() => onSelectTab(item.tab)}
                  className={`flex items-center space-x-2 px-3 py-2 rounded-md text-xs font-medium transition-colors ${
                    isActive
                      ? "bg-blue-600 text-white shadow-sm"
                      : "text-gray-300 hover:bg-gray-800 hover:text-white"
                  }`}
                >
                  <Icon className="w-4 h-4" />
                  <span>{item.label}</span>
                </button>
              );
            })}
          </nav>
        </div>
      </div>
    </header>
  );
};
