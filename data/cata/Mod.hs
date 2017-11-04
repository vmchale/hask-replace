{-# LANGUAGE CPP             #-}
{-# LANGUAGE TemplateHaskell #-}

-- module NotMod where

module Mod (function, makeBaseFunctor) where

-- import           Data.Functor.Foldable
import           Data.Functor.Foldable    (ListF (..), cata)
import           Data.Functor.Foldable.TH (makeBaseFunctor)

function :: (Num a) => [a] -> a
function = cata a where
    a Nil         = 0
    a (Cons x xs) = x + xs
