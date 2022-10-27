# yahoo2redists
Small utility: it retrieves historical quotes from yahoo finance and inserts them in a redis timeseries server. 

# Context

This script is part of a larger ecosystem of stand-alone applications serving the purpose of collecting and analysing financial data, namely **Archimedes**.
It's a porting in rust of its homologous written in python.

## Technology

A [**Redis**](https://github.com/redis/redis) node (with its module [**Redis TimeSeries**](https://github.com/RedisTimeSeries/RedisTimeSeries) enabled) is needed to successfully run the script.

The script, at this stage, expects to find the name of the tickers we are interested in, with this hard-coded prefix: `"MARKET:METADATA:STOCKS:*"`. 
It also expects to find, for each ticker, a `bool` value under the key `"MARKET:METADATA:STOCKS:*:attivo"` indicating whether the ticker is valid or we are not anymore interested in it (e.g.: it has been delisted).

Another script takes care of those keys.

# TO DOs

Future changes:

* parallel download;

* error management;

* elimination of hard coded values;

* notification of successful / unsuccessful retrieval to **Archimedes**;

* pre-check for de-listed stocks and update of tickers list.
