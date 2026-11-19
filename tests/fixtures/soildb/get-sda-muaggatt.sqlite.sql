SELECT muaggatt.mukey, muaggatt.* FROM legend INNER JOIN mapunit ON mapunit.lkey = legend.lkey INNER JOIN muaggatt ON mapunit.mukey = muaggatt.mukey WHERE mapunit.mukey IN ('1857266')
