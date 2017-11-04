{-# LANGUAGE CPP #-}
{-# OPTIONS_GHC -fno-warn-missing-import-lists #-}
{-# OPTIONS_GHC -fno-warn-implicit-prelude #-}
module Paths_cata (
    version,
    getBinDir, getLibDir, getDynLibDir, getDataDir, getLibexecDir,
    getDataFileName, getSysconfDir
  ) where

import qualified Control.Exception as Exception
import Data.Version (Version(..))
import System.Environment (getEnv)
import Prelude

#if defined(VERSION_base)

#if MIN_VERSION_base(4,0,0)
catchIO :: IO a -> (Exception.IOException -> IO a) -> IO a
#else
catchIO :: IO a -> (Exception.Exception -> IO a) -> IO a
#endif

#else
catchIO :: IO a -> (Exception.IOException -> IO a) -> IO a
#endif
catchIO = Exception.catch

version :: Version
version = Version [0,1,0,0] []
bindir, libdir, dynlibdir, datadir, libexecdir, sysconfdir :: FilePath

bindir     = "/home/vanessa/.cabal/bin"
libdir     = "/home/vanessa/.cabal/lib/x86_64-linux-ghc-8.2.1/cata-0.1.0.0-inplace"
dynlibdir  = "/home/vanessa/.cabal/lib/x86_64-linux-ghc-8.2.1"
datadir    = "/home/vanessa/.cabal/share/x86_64-linux-ghc-8.2.1/cata-0.1.0.0"
libexecdir = "/home/vanessa/.cabal/libexec/x86_64-linux-ghc-8.2.1/cata-0.1.0.0"
sysconfdir = "/home/vanessa/.cabal/etc"

getBinDir, getLibDir, getDynLibDir, getDataDir, getLibexecDir, getSysconfDir :: IO FilePath
getBinDir = catchIO (getEnv "cata_bindir") (\_ -> return bindir)
getLibDir = catchIO (getEnv "cata_libdir") (\_ -> return libdir)
getDynLibDir = catchIO (getEnv "cata_dynlibdir") (\_ -> return dynlibdir)
getDataDir = catchIO (getEnv "cata_datadir") (\_ -> return datadir)
getLibexecDir = catchIO (getEnv "cata_libexecdir") (\_ -> return libexecdir)
getSysconfDir = catchIO (getEnv "cata_sysconfdir") (\_ -> return sysconfdir)

getDataFileName :: FilePath -> IO FilePath
getDataFileName name = do
  dir <- getDataDir
  return (dir ++ "/" ++ name)
