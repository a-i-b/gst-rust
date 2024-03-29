/*
* (C) Copyright 2014 Kurento (http://kurento.org/)
*
* All rights reserved. This program and the accompanying materials
* are made available under the terms of the GNU Lesser General Public License
* (LGPL) version 2.1 which accompanies this distribution, and is available at
* http://www.gnu.org/licenses/lgpl-2.1.html
*
* This library is distributed in the hope that it will be useful,
* but WITHOUT ANY WARRANTY; without even the implied warranty of
* MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
* Lesser General Public License for more details.
*
*/

function getopts(args, opts)
{
  var result = opts.default || {};
  args.replace(
      new RegExp("([^?=&]+)(=([^&]*))?", "g"),
      function($0, $1, $2, $3) { result[$1] = $3; });

  return result;
};

var args = getopts(location.search,
{
  default:
  {
    ws_uri: 'ws://' + location.hostname + ':8888/kurento',
    ice_servers: undefined
  }
});

if (args.ice_servers) {
  console.log("Use ICE servers: " + args.ice_servers);
  kurentoUtils.WebRtcPeer.prototype.server.iceServers = JSON.parse(args.ice_servers);
} else {
  console.log("Use freeice")
}


window.addEventListener('load', function(){
  console = new Console('console', console);
	var videoOutput = document.getElementById('videoOutput');
	var address = document.getElementById('address');
	address.value = 'rtsp://host.docker.internal:8554/test';
  var pipeline;
  var webRtcPeer;

  startButton = document.getElementById('start');
  startButton.addEventListener('click', start);

  stopButton = document.getElementById('stop');
  stopButton.addEventListener('click', stop);

  function start() {
  	if(!address.value){
  	  window.alert("You must set the video source URL first");
  	  return;
  	}
  	address.disabled = true;
  	showSpinner(videoOutput);
    var options = {
      remoteVideo : videoOutput
    };
    webRtcPeer = kurentoUtils.WebRtcPeer.WebRtcPeerRecvonly(options,
      function(error){
        if(error){
          return console.error(error);
        }
        webRtcPeer.generateOffer(onOffer);
        webRtcPeer.peerConnection.addEventListener('iceconnectionstatechange', function(event){
          if(webRtcPeer && webRtcPeer.peerConnection){
            console.log("oniceconnectionstatechange -> " + webRtcPeer.peerConnection.iceConnectionState);
            console.log('icegatheringstate -> ' + webRtcPeer.peerConnection.iceGatheringState);
          }
        });
    });
  }

  function onOffer(error, sdpOffer){
    if(error) return onError(error);

  	kurentoClient(args.ws_uri, function(error, kurentoClient) {
  		if(error) return onError(error);

  		kurentoClient.create("MediaPipeline", function(error, p) {
  			if(error) return onError(error);

  			pipeline = p;

  			pipeline.create("PlayerEndpoint", {networkCache: 0, uri: address.value, useEncodedMedia: true}, function(error, player){
  			  if(error) return onError(error);

  			  pipeline.create("WebRtcEndpoint", function(error, webRtcEndpoint){
  				if(error) return onError(error);

          setIceCandidateCallbacks(webRtcEndpoint, webRtcPeer, onError);

          sdpOffer = removeOther(sdpOffer, "108")
  				webRtcEndpoint.processOffer(sdpOffer, function(error, sdpAnswer){
  					if(error) return onError(error);

            webRtcEndpoint.gatherCandidates(onError);

  					webRtcPeer.processAnswer(sdpAnswer);
  				});

  				player.connect(webRtcEndpoint, function(error){
  					if(error) return onError(error);

  					console.log("PlayerEndpoint-->WebRtcEndpoint connection established");

  					player.play(function(error){
  					  if(error) return onError(error);

  					  console.log("Player playing ...");
  					});
  				});
  			});
  			});
  		});
  	});
  }

  function stop() {
    address.disabled = false;
    if (webRtcPeer) {
      webRtcPeer.dispose();
      webRtcPeer = null;
    }
    if(pipeline){
      pipeline.release();
      pipeline = null;
    }
    hideSpinner(videoOutput);
  }

});

function setIceCandidateCallbacks(webRtcEndpoint, webRtcPeer, onError){
  webRtcPeer.on('icecandidate', function(candidate){
    console.log("Local icecandidate " + JSON.stringify(candidate));

    candidate = kurentoClient.register.complexTypes.IceCandidate(candidate);

    webRtcEndpoint.addIceCandidate(candidate, onError);

  });
  webRtcEndpoint.on('OnIceCandidate', function(event){
    var candidate = event.candidate;

    console.log("Remote icecandidate " + JSON.stringify(candidate));

    webRtcPeer.addIceCandidate(candidate, onError);
  });
}

function onError(error) {
  if(error)
  {
    console.error(error);
    stop();
  }
}

function showSpinner() {
	for (var i = 0; i < arguments.length; i++) {
		arguments[i].poster = 'img/transparent-1px.png';
		arguments[i].style.background = "center transparent url('img/spinner.gif') no-repeat";
	}
}

function hideSpinner() {
	for (var i = 0; i < arguments.length; i++) {
		arguments[i].src = '';
		arguments[i].poster = 'img/webrtc.png';
		arguments[i].style.background = '';
	}
}

function removeOther(sdp, id) {
  var resSDP = sdp;
  var sdpLines = splitLines(sdp);
  sdp.split('\n').forEach(function(line) {
    for (var i = 0; i < sdpLines.videoCodecNumbers.length; ++i) {
      if(sdpLines.videoCodecNumbers[i] != id) {
        if (line.indexOf(':' + sdpLines.videoCodecNumbers[i]) != -1) {
          resSDP = resSDP.replace(line+'\n', '');
        }     
      }
    }
  });

  return resSDP;
}

function splitLines(sdp) {
  var info = {};
  sdp.split('\n').forEach(function(line) {
      if (line.indexOf('m=video') === 0) {
          info.videoCodecNumbers = [];
          line.split('SAVPF')[1].split(' ').forEach(function(codecNumber) {
              codecNumber = codecNumber.trim();
              if (!codecNumber || !codecNumber.length) return;
              info.videoCodecNumbers.push(codecNumber);
              info.videoCodecNumbersOriginal = line;
          });
      }

      if (line.indexOf('VP8/90000') !== -1 && !info.vp8LineNumber) {
          info.vp8LineNumber = line.replace('a=rtpmap:', '').split(' ')[0];
      }

      if (line.indexOf('VP9/90000') !== -1 && !info.vp9LineNumber) {
          info.vp9LineNumber = line.replace('a=rtpmap:', '').split(' ')[0];
      }

      if (line.indexOf('H264/90000') !== -1 && !info.h264LineNumber) {
          info.h264LineNumber = line.replace('a=rtpmap:', '').split(' ')[0];
      }
  });

  return info;
}

/**
 * Lightbox utility (to display media pipeline image in a modal dialog)
 */
$(document).delegate('*[data-toggle="lightbox"]', 'click', function(event) {
	event.preventDefault();
	$(this).ekkoLightbox();
});
