# Firmware Dumps and Binary Images (`firmware_and_images/`)

Este diretório armazena os dumps brutos de memória e imagens de armazenamento do Panasonic DMR-EH55 utilizados pelas ferramentas de análise forense e engenharia reversa do `panadiga`.

> [!NOTE]
> Por motivos de restrições de tamanho do GitHub (arquivos > 100 MB) e proteção de propriedade intelectual/direitos autorais de terceiros, os dumps brutos proprietários não são versionados diretamente no repositório Git público.
> Ao trabalhar com o gravador, coloque seus dumps obtidos em bancada nesta pasta com os nomes descritos abaixo.

---

## 📁 Arquivos Esperados

### 1. Dump do Disco Rígido (`panasonic DMR EH-55 Firmware hdd_100.bin`)
* **Descrição:** Leitura física dos primeiros 100 MB do disco rígido original PATA/IDE (superbloco MEIHDFS, tabelas espelho, journal e partições de firmware).
* **Tamanho exato:** `104.857.600 bytes` (204.800 setores LBA de 512 bytes).
* **Hash MD5:** `ef4453fc8651903b0a4b0c1873d950d4`
* **Hash SHA-256:** `9d292d5b46f190775f359e30af11d074021fb321f040bf701d0ecf0f2056db00`

### 2. Contêiner Óptico de Atualização (`PANA_DVD.FRM`)
* **Descrição:** Imagem oficial de atualização óptica de firmware para CD-R/DVD-RAM da Panasonic.
* **Tamanho exato:** `7.531.744 bytes`.
* **Hash MD5:** `682bf55cf73e728790351798ec4f58c4`
* **Hash SHA-256:** `fc5e96f2648ec823126f74a0da72c57eb430d4a974917ae69ef112bb67fa7d34`

### 3. Dump da Flash Spansion S29JL064H (`IC55007.bin`)
* **Descrição:** Leitura de 8 MB da memória NOR Flash TSOP-48 contendo o bootloader IPL e a partição NVRAM/EEMUL corrompida (`U99`).
* **Tamanho exato:** `8.388.608 bytes`.
